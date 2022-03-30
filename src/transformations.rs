use crate::gadget::*;
use egui::{Ui};

pub struct AbsGadget {
    inp: Parameter,
    out: Parameter,
    instance_name: String,
}

impl AbsGadget {
    pub fn new(name: &str) -> AbsGadget {
        AbsGadget {
            inp: Parameter::new("inp", 0.0),
            out: Parameter::new("out", 0.0),
            instance_name: name.to_owned(),
        }
    }
}

impl GadgetUI for AbsGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for AbsGadget {}

impl Gadget for AbsGadget {
    fn name(&self) -> &'static str {
        "ABS"
    }
    fn get_instance_name(&self) -> String {
        self.instance_name.to_owned()
    }
    fn par(&self, i: usize) -> &Parameter {
        match i {
            0 => &self.inp,
            1 => &self.out,
            _ => panic!("Invalid parameter number in AbsGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.inp,
            1 => &mut self.out,
            _ => panic!("Invalid parameter number in AbsGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        2
    }
    #[inline]
    fn run(&mut self) {
        *self.out = (*self.inp).abs();
    }
}

pub struct DoubleAbsGadget {
    x: Parameter,
    y: Parameter,
    out: Parameter,
    instance_name: String,
}

impl DoubleAbsGadget {
    pub fn new(name: &str) -> DoubleAbsGadget {
        DoubleAbsGadget {
            x: Parameter::new("x", 0.0),
            y: Parameter::new("y", 0.0),
            out: Parameter::new("out", 0.0),
            instance_name: name.to_owned(),
        }
    }
}

impl GadgetUI for DoubleAbsGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for DoubleAbsGadget {}

impl Gadget for DoubleAbsGadget {
    fn name(&self) -> &'static str {
        "DABS"
    }
    fn get_instance_name(&self) -> String {
        self.instance_name.to_owned()
    }
    fn par(&self, i: usize) -> &Parameter {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.out,
            _ => panic!("Invalid parameter number in DoubleAbsGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.out,
            _ => panic!("Invalid parameter number in DoubleAbsGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        3
    }
    #[inline]
    fn run(&mut self) {
        *self.out = (*self.x).abs() - (*self.y).abs();
    }
}


pub struct AmplitudePhaseGadget {
    x: Parameter,
    y: Parameter,
    amplitude: Parameter,
    phase: Parameter,
    instance_name: String,
}

impl AmplitudePhaseGadget {
    pub fn new(name: &str) -> AmplitudePhaseGadget {
        AmplitudePhaseGadget {
            x: Parameter::new("x", 0.0),
            y: Parameter::new("y", 0.0),
            amplitude: Parameter::new("amplitude", 0.0),
            phase: Parameter::new("phase", 0.0),
            instance_name: name.to_owned(),
        }
    }
}

impl GadgetUI for AmplitudePhaseGadget {
    fn gui(&mut self, link: &mut Option<String>, ui: &mut Ui) {
        gadget_gui(self, link, ui);
    }
}

impl GadgetWithUI for AmplitudePhaseGadget {}

impl Gadget for AmplitudePhaseGadget {
    fn name(&self) -> &'static str {
        "DABS"
    }
    fn get_instance_name(&self) -> String {
        self.instance_name.to_owned()
    }
    fn par(&self, i: usize) -> &Parameter {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.amplitude,
            3 => &self.phase,
            _ => panic!("Invalid parameter number in AmplitudePhaseGadget"),
        }
    }
    fn par_mut(&mut self, i: usize) -> &mut Parameter {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.amplitude,
            3 => &mut self.phase,
            _ => panic!("Invalid parameter number in AmplitudePhaseGadget"),
        }
    }
    fn parameter_count(&self) -> usize {
        4
    }
    #[inline]
    fn run(&mut self) {
        *self.amplitude = ((*self.x)*(*self.x) + (*self.y)*(*self.y)).sqrt();
        *self.phase = (*self.y).atan2(*self.x)/std::f32::consts::FRAC_PI_2;
    }
}
