use std::ops::{Deref, DerefMut};
use egui::{Ui};

pub const SAMPLERATE: u32 = 48000;
pub const DT: f32 = 1.0 / (SAMPLERATE as f32);


#[derive(Debug, Clone)]
pub enum Link {
    Value(f32),
    Link(String),
}

pub struct Parameter {
    pub value: *mut f32,
    pub name: &'static str,
    pub link: Link,
}

pub const ZERO: *mut f32 = std::ptr::null_mut();

impl Parameter {
    pub fn new(name: &'static str, value:f32) -> Parameter {
        Parameter {
            value: ZERO,
            name: name,
            link: Link::Value(value),
        }
    }
    pub fn set_value(&mut self, value: f32) {
        self.link = Link::Value(value)
    }
    pub fn set_link(&mut self, value: &str) {
        self.link = Link::Link(value.to_owned())
    }
    pub fn is_unbound(&self) -> bool {
        //println!("value: {:?} ZERO: {:?}",self.value, ZERO);
        std::ptr::eq(self.value, ZERO)
    }
    pub fn is_free(&self) -> bool {
        match self.link {
            Link::Value(_) => true,
            Link::Link(_) => false,
        }
    }
    pub fn bind(&mut self, buffer: *mut f32) {
        self.value = buffer;
    }
    pub fn unbind(&mut self) {
        self.value = ZERO;
    }
}

impl Deref for Parameter {
    type Target = f32;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl DerefMut for Parameter {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

pub trait Gadget {
    fn name(&self) -> &'static str;
    fn get_instance_name(&self) -> String;
    fn par(&self, i: usize) -> &Parameter;
    fn par_mut(&mut self, i: usize) -> &mut Parameter;
    fn parameter_count(&self) -> usize;
    fn run(&mut self);

    fn parameter_names(&self) -> Vec<String> {
        let instance_name = self.get_instance_name();
        let mut p = Vec::with_capacity(self.parameter_count());
        for i in 0..self.parameter_count() {
            p.push(format!("{}: {}", instance_name, self.par(i).name))
        }
        p
    }
    fn parameter(&self, name: &str) -> Option<&Parameter> {
        for (i, parameter_name) in self.parameter_names().iter().enumerate() {
            if parameter_name == name {
                return Some(self.par(i));
            }
        }
        None
    }
    fn parameter_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        for (i, parameter_name) in self.parameter_names().iter().enumerate() {
            if parameter_name == name {
                return Some(self.par_mut(i));
            }
        }
        None
    }
    fn free_parameter_count(&self) -> usize {
        (0..self.parameter_count())
            .filter(|x| self.par(*x).is_free())
            .count()
    }
}

pub trait GadgetUI {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui);
}

pub fn gadget_gui<G: Gadget>(gadget: &mut G, link: &mut Option<String>, ui: &mut Ui) {
    ui.collapsing(gadget.get_instance_name(), |ui| {
        let pnames = gadget.parameter_names();
        egui::Grid::new(format!("_Grid_{}", gadget.get_instance_name())).show(ui, |ui| {
            //ui.colored_label(egui::Color32::LIGHT_BLUE, gadget.get_instance_name());
            //ui.end_row();

            for i in 0..gadget.parameter_count() {
                let p = gadget.par_mut(i);
                ui.label(p.name);
                match p.link.clone() {
                    Link::Link(link) => {
                        ui.label(link);
                        if ui.button("Unlink").clicked() {
                            p.set_value(0.0);
                        }
                    }
                    Link::Value(x) => {
                        let mut value = x;
                        ui.add(egui::widgets::DragValue::new(&mut value));
                        p.link = Link::Value(value);
                        if ui.button("Select").clicked() {
                            *link = Some(pnames[i].to_owned());
                        }
                    }
                }
                if let Some(name) = link {
                    if ui.button(name.to_owned()).clicked() {
                        p.set_link(name);
                    }
                }
                ui.end_row();
            }
        })
    });
}

pub trait GadgetWithUI: Gadget + GadgetUI {}
pub struct GadgetContainer {
    pub container: Vec<Box<dyn GadgetWithUI>>,
}
impl GadgetContainer {
    pub fn new() -> Self {
        GadgetContainer {
            container: Vec::new(),
        }
    }
}
impl GadgetUI for GadgetContainer {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        egui::Grid::new("GadgetContainer").show(ui, |ui| {
            for gadget in self.container.iter_mut() {
                gadget.gui(link, ui);
                ui.end_row();
            }
        });
    }
}

impl Gadget for GadgetContainer {
    fn name(&self) -> &'static str {
        "container"
    }
    fn get_instance_name(&self) -> String {
        self.name().to_string()
    }
    fn parameter_names(&self) -> Vec<String> {
        let mut p = Vec::with_capacity(self.parameter_count());
        for g in &self.container {
            for n in g.parameter_names() {
                p.push(n);
            }
        }
        p
    }
    fn par(&self, i: usize) -> &Parameter {
        let mut count = 0;
        for gadget in self.container.iter() {
            if i >= count && i < count + gadget.parameter_count() {
                return gadget.par(i - count);
            }
            count += gadget.parameter_count();
        }
        panic!(
            "Invalid parameter number {} in container of size {}",
            i,
            self.container.len()
        );
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        let mut count = 0;
        for gadget in self.container.iter_mut() {
            if i >= count && i < count + gadget.parameter_count() {
                return gadget.par_mut(i - count);
            }
            count += gadget.parameter_count();
        }
        panic!("Invalid parameter number in container");
    }
    fn parameter_count(&self) -> usize {
        self.container.iter().map(|x| x.parameter_count()).sum()
    }
    fn run(&mut self) {
        for gadget in self.container.iter_mut() {
            gadget.run();
        }
    }
}
