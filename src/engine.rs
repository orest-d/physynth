use crate::gadget::*;
use std::time::Duration;
use egui::{Ui};
use rodio::{source::Source};

pub struct OutputGadget {
    output: Parameter,
}

impl OutputGadget {
    pub fn new() -> OutputGadget {
        OutputGadget {
            output: Parameter::new("OUT", 0.0),
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

pub struct Engine<G: Gadget> {
    pub gadget: G,
    pub buffer: Vec<f32>,
    pub output: *mut f32,
}

impl<G: Gadget> Engine<G> {
    pub fn new(gadget: G) -> Self {
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

    pub fn bind(&mut self) {
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
    pub fn run(&mut self) {
        self.gadget.run();
    }
    #[inline]
    pub fn out(&self) -> f32 {
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
