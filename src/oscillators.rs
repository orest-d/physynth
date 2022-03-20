use crate::gadget::*;
use egui::{Ui};
use std::f32::consts::PI;

pub struct DampedOscillatorGadget {
    frequency: Parameter,
    x: Parameter,
    xs: Parameter,
    y: Parameter,
    ys: Parameter,
    damp: Parameter,
    m: Parameter,
    instance_name: String,
}

impl DampedOscillatorGadget {
    pub fn new(name: &str) -> DampedOscillatorGadget {
        DampedOscillatorGadget {
            frequency: Parameter::new("frequency", 440.0),
            x: Parameter::new("x", 1.0),
            xs: Parameter::new("xs", 0.0),
            y: Parameter::new("y", 0.0),
            ys: Parameter::new("ys", 0.0),
            damp: Parameter::new("damp",1.0),
            m: Parameter::new("m",1.0),
            instance_name: name.to_owned(),
        }
    }
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
            2 => &self.xs,
            3 => &self.y,
            4 => &self.ys,
            5 => &self.damp,
            6 => &self.m,
            _ => panic!("Invalid parameter number in DampedOscillatorGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.frequency,
            1 => &mut self.x,
            2 => &mut self.xs,
            3 => &mut self.y,
            4 => &mut self.ys,
            5 => &mut self.damp,
            6 => &mut self.m,
            _ => panic!("Invalid parameter number in DampedOscillatorGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        7
    }
    #[inline]
    fn run(&mut self) {
        let m = if *self.m > 0.0 { *self.m } else { 1.0 };
        let omega = 2.0 * PI * *self.frequency;
        let k = m * omega * omega;
        *self.y += -(k * *self.x + *self.damp * *self.y) * DT / m + *self.ys;
        *self.x += *self.y * DT + *self.xs;
    }
}

pub struct PowerOscillatorGadget {
    frequency: Parameter,
    x: Parameter,
    xs: Parameter,
    pwx: Parameter,
    y: Parameter,
    ys: Parameter,
    pwy: Parameter,
    damp: Parameter,
    m: Parameter,
    instance_name: String,
}

impl PowerOscillatorGadget {
    pub fn new(name: &str) -> PowerOscillatorGadget {
        PowerOscillatorGadget {
            frequency: Parameter::new("frequency", 440.0),
            x: Parameter::new("x", 1.0),
            xs: Parameter::new("xs", 0.0),
            pwx: Parameter::new("pwx", 0.0),
            y: Parameter::new("y", 0.0),
            ys: Parameter::new("ys", 0.0),
            pwy: Parameter::new("pwy", 0.0),
            damp: Parameter::new("damp",1.0),
            m: Parameter::new("m",1.0),
            instance_name: name.to_owned(),
        }
    }
    fn power(x:f32, p:f32)->f32{
        if x.abs()<1e-5{
            x
        }
        else{
            x*x.abs().powf(p)
        }
    }
}

impl GadgetUI for PowerOscillatorGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for PowerOscillatorGadget {}

impl Gadget for PowerOscillatorGadget {
    fn name(&self) -> &'static str {
        "PwO"
    }
    fn get_instance_name(&self) -> String {
        self.instance_name.to_owned()
    }
    fn par(&self, i: usize) -> &Parameter {
        match i {
            0 => &self.frequency,
            1 => &self.x,
            2 => &self.xs,
            3 => &self.pwx,
            4 => &self.y,
            5 => &self.ys,
            6 => &self.pwy,
            7 => &self.damp,
            8 => &self.m,
            _ => panic!("Invalid parameter number in PowerOscillatorGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.frequency,
            1 => &mut self.x,
            2 => &mut self.xs,
            3 => &mut self.pwx,
            4 => &mut self.y,
            5 => &mut self.ys,
            6 => &mut self.pwy,
            7 => &mut self.damp,
            8 => &mut self.m,
            _ => panic!("Invalid parameter number in PowerOscillatorGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        9
    }

    #[inline]
    fn run(&mut self) {
        let m = if *self.m > 0.0 { *self.m } else { 1.0 };
        let omega = 2.0 * PI * *self.frequency;
        let k = m * omega * omega;
        *self.y += -(k * PowerOscillatorGadget::power(*self.x,*self.pwx) + *self.damp * PowerOscillatorGadget::power(*self.y,*self.pwy)) * DT / m + *self.ys;
        *self.x += *self.y * DT + *self.xs;
    }
}
