use macroquad::prelude::*;
extern crate midir;

use std::error::Error;

use midir::{Ignore, MidiInput, MidiOutput};
use rodio::OutputStream;

pub mod engine;
pub mod gadget;
pub mod oscillators;
pub mod transformations;
use egui::plot::{Line, Plot, Value, Values};
use engine::*;
use gadget::*;
use oscillators::*;
use transformations::*;

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
    //let source = DampedOscillator::new(440.0);
    //    stream_handle.play_raw(source).unwrap();
    let mut container = GadgetContainer::new();
    container.container.push(Box::new(OutputGadget::new()));
    container
        .container
        .push(Box::new(DampedOscillatorGadget::new("Osc")));
    let mut engine = Engine::new(container);
    engine.bind();
    let mut link: Option<String> = None;
    let mut buffer = Vec::with_capacity(100000);

    loop {
        clear_background(BLACK);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Φ Synth").show(egui_ctx, |ui| {
                ui.label("placeholder");
                ui.label(check_midi().unwrap());
            });
            egui::Window::new("Synth").show(egui_ctx, |ui| {
                if ui.button("Play").clicked() {
                    (&mut engine).bind();
                    (&mut buffer).clear();
                    for x in (&mut engine).take(100000) {
                        (&mut buffer).push(x);
                    }
                    let source = rodio::buffer::SamplesBuffer::new(1, 44100, buffer.as_slice());
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
                if ui.button("Add PwOsc").clicked() {
                    engine
                        .gadget
                        .container
                        .push(Box::new(PowerOscillatorGadget::new(&format!(
                            "PwOsc{}",
                            engine.gadget.container.len() - 1
                        ))));
                }
                if ui.button("Add Abs").clicked() {
                    engine
                        .gadget
                        .container
                        .push(Box::new(AbsGadget::new(&format!(
                            "ABS{}",
                            engine.gadget.container.len() - 1
                        ))));
                }
                if ui.button("Add DAbs").clicked() {
                    engine
                        .gadget
                        .container
                        .push(Box::new(DoubleAbsGadget::new(&format!(
                            "DABS{}",
                            engine.gadget.container.len() - 1
                        ))));
                }
                if ui.button("Add A/P").clicked() {
                    engine
                        .gadget
                        .container
                        .push(Box::new(AmplitudePhaseGadget::new(&format!(
                            "AP{}",
                            engine.gadget.container.len() - 1
                        ))));
                }
            });
            egui::Window::new("Plot").show(egui_ctx, |ui| {
                if !(&buffer).is_empty() {
                    let line = Line::new(Values::from_values_iter(
                        (&buffer)
                            .iter().take(5000)
                            .enumerate()
                            .map(|(i, &x)| Value::new((i as f64) * (DT as f64), x as f64)),
                    ));
                    
                    Plot::new("my_plot")
                        .view_aspect(2.0)
                        .show(ui, |plot_ui| plot_ui.line(line));
                        
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
        let p = Parameter::new("test", 0.0);
        assert!(p.is_unbound());
    }
    #[test]
    fn test_bind_parameter() {
        let mut buffer: f32 = 123.0;
        let mut p = Parameter::new("test", 0.0);
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
