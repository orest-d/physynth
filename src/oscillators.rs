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
            _ => panic!("Invalid parameter number in DampedOscillatorGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        6
    }
    #[inline]
    fn run(&mut self) {
        let omega = 2.0 * PI * *self.frequency;
        *self.x += *self.y * omega * DT + *self.ys * DT;
        *self.y += -(*self.x + 2.0* *self.damp * *self.y) * omega * DT + *self.xs*DT;
    }
}

pub struct PowerOscillatorGadget {
    frequency: Parameter,
    x: Parameter,
    xs: Parameter,
    y: Parameter,
    ys: Parameter,
    damp: Parameter,
    power: Parameter,
    alpha: Parameter,
    instance_name: String,
}

impl PowerOscillatorGadget {
    pub fn new(name: &str) -> PowerOscillatorGadget {
        PowerOscillatorGadget {
            frequency: Parameter::new("frequency", 440.0),
            x: Parameter::new("x", 1.0),
            xs: Parameter::new("xs", 0.0),
            y: Parameter::new("y", 0.0),
            ys: Parameter::new("ys", 0.0),
            damp: Parameter::new("damp",1.0),
            power: Parameter::new("power", 0.0),
            alpha: Parameter::new("alpha", 0.0),
            instance_name: name.to_owned(),
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
            3 => &self.y,
            4 => &self.ys,
            5 => &self.damp,
            6 => &self.power,
            7 => &self.alpha,
            _ => panic!("Invalid parameter number in PowerOscillatorGadget"),
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
            6 => &mut self.power,
            7 => &mut self.alpha,
            _ => panic!("Invalid parameter number in PowerOscillatorGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        8
    }

    #[inline]
    fn run(&mut self) {
        let omega = 2.0 * PI * *self.frequency;
        let ca = (*self.alpha * PI).cos();
        let sa = (*self.alpha * PI).sin();
        let wx = *self.x * ca +*self.y * sa;
        let wy = *self.y * ca -*self.x * sa;
        let pwx = wx.powf(*self.power).abs()*wx;
        let pwy = wy.powf(*self.power).abs()*wy;
        let n = (pwx*pwx + pwy*pwy).sqrt().max(0.01);
        let gx = pwx*ca/n - pwy*sa/n;
        let gy = pwx*sa/n + pwy*ca/n;


        *self.x += gy * omega * DT + *self.ys * DT;
        *self.y += -(gx + 2.0* *self.damp * *self.y) * omega * DT + *self.xs*DT;
    }
}
