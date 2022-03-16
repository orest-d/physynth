use macroquad::prelude::*;
extern crate midir;

use std::error::Error;
use std::f32::consts::PI;
use std::hash::Hash;
use std::time::Duration;

use egui::{RichText, Ui};
use std::ops::{Deref, DerefMut};

use midir::{Ignore, MidiInput, MidiOutput};
use rodio::{source::Source, OutputStream};

const SAMPLERATE: u32 = 48000;
const DT: f32 = 1.0 / (SAMPLERATE as f32);

#[derive(Debug, Clone)]
enum Link {
    Value(f32),
    Link(String),
}

struct Parameter {
    value: *mut f32,
    name: &'static str,
    link: Link,
}

const ZERO: *mut f32 = std::ptr::null_mut();

impl Parameter {
    fn new(name: &'static str) -> Parameter {
        Parameter {
            value: ZERO,
            name: name,
            link: Link::Value(0.0),
        }
    }
    fn set_value(&mut self, value: f32) {
        self.link = Link::Value(value)
    }
    fn set_link(&mut self, value: &str) {
        self.link = Link::Link(value.to_owned())
    }
    fn is_unbound(&self) -> bool {
        //println!("value: {:?} ZERO: {:?}",self.value, ZERO);
        std::ptr::eq(self.value, ZERO)
    }
    fn is_free(&self) -> bool {
        match self.link {
            Link::Value(_) => true,
            Link::Link(_) => false,
        }
    }
    fn bind(&mut self, buffer: *mut f32) {
        self.value = buffer;
    }
    fn unbind(&mut self) {
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

struct DampedOscillatorGadget {
    frequency: Parameter,
    x: Parameter,
    y: Parameter,
    damp: Parameter,
    m: Parameter,
    instance_name: String,
}

impl DampedOscillatorGadget {
    fn new(name: &str) -> DampedOscillatorGadget {
        DampedOscillatorGadget {
            frequency: Parameter::new("frequency"),
            x: Parameter::new("x"),
            y: Parameter::new("y"),
            damp: Parameter::new("damp"),
            m: Parameter::new("m"),
            instance_name: name.to_owned(),
        }
    }
}
trait Gadget {
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

trait GadgetUI {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui);
}

fn gadget_gui<G: Gadget>(gadget: &mut G, link: &mut Option<String>, ui: &mut Ui) {
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
                            p.link = Link::Value(0.0);
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
                        p.link = Link::Link(name.to_owned());
                    }
                }
                ui.end_row();
            }
        })
    });
}

impl GadgetUI for DampedOscillatorGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for DampedOscillatorGadget {}

impl Gadget for DampedOscillatorGadget {
    fn name(&self) -> &'static str {
        "DO"
    }
    fn get_instance_name(&self) -> String {
        self.instance_name.to_owned()
    }
    fn par(&self, i: usize) -> &Parameter {
        match i {
            0 => &self.frequency,
            1 => &self.x,
            2 => &self.y,
            3 => &self.damp,
            4 => &self.m,
            _ => panic!("Invalid parameter number in DampedOscillatorGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.frequency,
            1 => &mut self.x,
            2 => &mut self.y,
            3 => &mut self.damp,
            4 => &mut self.m,
            _ => panic!("Invalid parameter number in DampedOscillatorGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        5
    }
    #[inline]
    fn run(&mut self) {
        let m = if (*self.m > 0.0) { *self.m } else { 1.0 };
        let omega = 2.0 * PI * *self.frequency;
        let k = m * omega * omega;
        *self.y += -(k * *self.x + *self.damp * *self.y) * DT / m;
        *self.x += *self.y * DT;
    }
}

trait GadgetWithUI: Gadget + GadgetUI {}
struct GadgetContainer {
    container: Vec<Box<dyn GadgetWithUI>>,
}
impl GadgetContainer {
    fn new() -> Self {
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

struct OutputGadget {
    output: Parameter,
}

impl OutputGadget {
    fn new() -> OutputGadget {
        OutputGadget {
            output: Parameter::new("OUT"),
        }
    }
}

impl GadgetUI for OutputGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for OutputGadget {}

impl Gadget for OutputGadget {
    fn name(&self) -> &'static str {
        "Output"
    }
    fn get_instance_name(&self) -> String {
        "Output".to_owned()
    }
    fn parameter_names(&self) -> Vec<String> {
        vec!["OUT".to_owned()]
    }

    fn par(&self, i: usize) -> &Parameter {
        if i != 0 {
            panic!("Invalid parameter number {} in output", i);
        }
        &self.output
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        if i != 0 {
            panic!("Invalid parameter number {} in output", i);
        }
        &mut self.output
    }
    fn parameter_count(&self) -> usize {
        1
    }
    fn run(&mut self) {}
}

struct Engine<G: Gadget> {
    gadget: G,
    buffer: Vec<f32>,
    output: *mut f32,
}

impl<G: Gadget> Engine<G> {
    fn new(gadget: G) -> Self {
        Self {
            gadget: gadget,
            buffer: Vec::new(),
            output: ZERO,
        }
    }
    fn root_parameter_name(&self, name: &str) -> String {
        match self.gadget.parameter(name) {
            Some(&Parameter {
                link: Link::Link(ref lnk),
                ..
            }) => self.root_parameter_name(lnk),
            _ => name.to_owned(),
        }
    }
    /*
    fn root_parameter_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        let name = self.root_parameter_name(name);
        self.gadget.parameter_mut(&name)
    }
    */
    fn root_parameter_pointer(&self, name: &str) -> *mut f32 {
        let name = self.root_parameter_name(name);
        self.gadget
            .parameter(&name)
            .expect(&format!("Parameter {} should be known", name))
            .value
    }

    fn bind(&mut self) {
        self.buffer.resize(self.gadget.free_parameter_count(), 0.0);
        let mut ptr = self.buffer.as_mut_ptr();
        for i in 0..self.gadget.parameter_count() {
            let p = self.gadget.par_mut(i);
            p.unbind();

            if let Link::Value(x) = p.link {
                p.bind(ptr);
                **p = x;
            }
            unsafe { ptr = ptr.add(1) };
        }
        for i in 0..self.gadget.parameter_count() {
            let p = self.gadget.par(i);
            if let Link::Link(ref x) = p.link {
                let ptr = self.root_parameter_pointer(x);
                self.gadget.par_mut(i).bind(ptr);
            }
        }
        let name = self.root_parameter_name("OUT");
        if let Some(p) = self.gadget.parameter(&name) {
            self.output = p.value;
        } else {
            self.output = ZERO;
        }
    }
    #[inline]
    fn run(&mut self) {
        self.gadget.run();
    }
    #[inline]
    fn out(&self) -> f32 {
        if std::ptr::eq(self.output, ZERO) {
            0.0
        } else {
            unsafe { *self.output }
        }
    }
}

impl<G: Gadget> Iterator for Engine<G> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.run();
        Some(self.out())
    }
}

impl<G: Gadget> Source for Engine<G> {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

struct DampedOscillator {
    //frequency: f32,
    x: f32,
    y: f32,
    damp: f32,
    m: f32,
    k: f32,
    //omega: f32,
}

impl DampedOscillator {
    /// The frequency of the sine.
    #[inline]
    pub fn new(frequency: f32) -> DampedOscillator {
        let omega = 2.0 * PI * frequency;
        let m = 1.0;
        let k = m * omega * omega;

        DampedOscillator {
            //frequency: frequency,
            x: 1.0,
            y: 0.0,
            damp: 0.5,
            m: m,
            k: k,
            //omega: omega,
        }
    }
}

impl Iterator for DampedOscillator {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.y += -(self.k * self.x + self.damp * self.y) * DT / self.m;
        self.x += self.y * DT;
        Some(self.x)
    }
}

impl Source for DampedOscillator {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Phi-Synth".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

fn check_midi() -> Result<String, Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir test input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir test output")?;

    let mut res = String::new();

    res = res + &format!("Available input ports:\n");
    for (i, p) in midi_in.ports().iter().enumerate() {
        res = res + &format!("{}: {}\n", i, midi_in.port_name(p)?);
    }
    res = res + &format!("\nAvailable output ports:\n");
    for (i, p) in midi_out.ports().iter().enumerate() {
        res = res + &format!("{}: {}\n", i, midi_out.port_name(p)?);
    }
    Ok(res)
}

#[macroquad::main(window_conf)]
async fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let source = DampedOscillator::new(440.0);
    //    stream_handle.play_raw(source).unwrap();
    let mut container = GadgetContainer::new();
    container.container.push(Box::new(OutputGadget::new()));
    container
        .container
        .push(Box::new(DampedOscillatorGadget::new("Osc")));
    let mut engine = Engine::new(container);
    engine.bind();
    let mut link: Option<String> = None;

    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Phi Synth").show(egui_ctx, |ui| {
                ui.label("placeholder");
                ui.label(check_midi().unwrap());
            });
            egui::Window::new("Synth").show(egui_ctx, |ui| {
                if ui.button("Play").clicked() {
                    (&mut engine).bind();
                    let mut buffer = Vec::with_capacity(100000);
                    for (i, x) in (&mut engine).take(100000).enumerate() {
                        buffer.push(x);
                    }
                    let mut source = rodio::buffer::SamplesBuffer::new(1, 44100, buffer);
                    stream_handle.play_raw(source).unwrap();
                }
                if let Some(text) = &link {
                    ui.label(format!("Link: {}", text));
                } else {
                    ui.label("No link");
                }
                engine.gadget.gui(&mut link, ui);
                if ui.button("Add Osc").clicked() {
                    engine
                        .gadget
                        .container
                        .push(Box::new(DampedOscillatorGadget::new(&format!(
                            "Osc{}",
                            engine.gadget.container.len() - 1
                        ))));
                }
            });
        });
        egui_macroquad::draw();
        next_frame().await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unbound() {
        let p = Parameter::new("test");
        assert!(p.is_unbound());
    }
    #[test]
    fn test_bind_parameter() {
        let mut buffer: f32 = 123.0;
        let mut p = Parameter::new("test");
        p.bind(&mut buffer);
        assert_eq!(*p, 123.0);
        *p = 456.0;
        assert_eq!(buffer, 456.0);
    }

    #[test]
    fn test_dpo() {
        let mut engine = Engine::new(DampedOscillatorGadget::new("Osc"));
        engine.bind();
        engine.gadget.run();
    }
}
