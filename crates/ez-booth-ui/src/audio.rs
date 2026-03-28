use leptos::set_timeout;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::{AudioContext, OscillatorType};

pub fn play_error_sound() -> Result<(), JsValue> {
    let context = Rc::new(AudioContext::new()?);
    let oscillator = match context.create_oscillator() {
        Ok(oscillator) => oscillator,
        Err(err) => {
            let _ = context.close();
            return Err(err);
        }
    };
    let gain = match context.create_gain() {
        Ok(gain) => gain,
        Err(err) => {
            let _ = context.close();
            return Err(err);
        }
    };

    let context_for_cleanup = Rc::clone(&context);
    let oscillator_for_cleanup = oscillator.clone();
    let gain_for_cleanup = gain.clone();

    let result = (|| -> Result<(), JsValue> {
        oscillator.set_type(OscillatorType::Triangle);
        oscillator.frequency().set_value(392.0);

        let now = context.current_time();
        gain.gain().set_value_at_time(0.0001, now)?;
        gain.gain().linear_ramp_to_value_at_time(0.16, now + 0.01)?;
        gain.gain()
            .exponential_ramp_to_value_at_time(0.0001, now + 0.16)?;

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&context.destination())?;

        oscillator.start()?;
        oscillator.stop_with_when(now + 0.16)?;

        Ok(())
    })();

    set_timeout(
        move || {
            let _ = oscillator_for_cleanup.disconnect();
            let _ = gain_for_cleanup.disconnect();
            let _ = context_for_cleanup.close();
        },
        Duration::from_millis(250),
    );

    result
}
