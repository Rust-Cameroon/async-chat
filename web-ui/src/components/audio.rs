use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorType};

/// Play a notification sound using Web Audio API
pub fn play_notification_sound() {
    if let Err(e) = play_notification_internal() {
        web_sys::console::log_1(&format!("Audio error: {:?}", e).into());
    }
}

fn play_notification_internal() -> Result<(), JsValue> {
    let context = AudioContext::new()?;
    
    // Create oscillator for the tone
    let oscillator = context.create_oscillator()?;
    let gain = context.create_gain()?;
    
    // Connect oscillator -> gain -> destination
    oscillator.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&context.destination())?;
    
    // Configure the sound - pleasant notification tone
    oscillator.set_type(OscillatorType::Sine);
    oscillator.frequency().set_value(880.0); // A5 note
    
    // Envelope: quick fade in, sustain, fade out
    let now = context.current_time();
    gain.gain().set_value_at_time(0.0, now)?;
    gain.gain().linear_ramp_to_value_at_time(0.3, now + 0.05)?; // Fade in
    gain.gain().linear_ramp_to_value_at_time(0.3, now + 0.1)?;  // Sustain
    gain.gain().linear_ramp_to_value_at_time(0.0, now + 0.3)?;  // Fade out
    
    // Play the sound
    oscillator.start()?;
    oscillator.stop_with_when(now + 0.3)?;
    
    Ok(())
}

/// Play a simple click sound for UI feedback
pub fn play_click_sound() {
    if let Err(_) = play_click_internal() {
        // Silently fail for click sounds
    }
}

fn play_click_internal() -> Result<(), JsValue> {
    let context = AudioContext::new()?;
    let oscillator = context.create_oscillator()?;
    let gain = context.create_gain()?;
    
    oscillator.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&context.destination())?;
    
    oscillator.set_type(OscillatorType::Sine);
    oscillator.frequency().set_value(1200.0);
    
    let now = context.current_time();
    gain.gain().set_value_at_time(0.1, now)?;
    gain.gain().linear_ramp_to_value_at_time(0.0, now + 0.05)?;
    
    oscillator.start()?;
    oscillator.stop_with_when(now + 0.05)?;
    
    Ok(())
}

/// Play a send message sound
pub fn play_send_sound() {
    if let Err(_) = play_send_internal() {
        // Silently fail
    }
}

fn play_send_internal() -> Result<(), JsValue> {
    let context = AudioContext::new()?;
    let oscillator = context.create_oscillator()?;
    let gain = context.create_gain()?;
    
    oscillator.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&context.destination())?;
    
    oscillator.set_type(OscillatorType::Sine);
    
    // Rising tone for "sent" feeling
    let now = context.current_time();
    oscillator.frequency().set_value_at_time(600.0, now)?;
    oscillator.frequency().linear_ramp_to_value_at_time(900.0, now + 0.1)?;
    
    gain.gain().set_value_at_time(0.15, now)?;
    gain.gain().linear_ramp_to_value_at_time(0.0, now + 0.15)?;
    
    oscillator.start()?;
    oscillator.stop_with_when(now + 0.15)?;
    
    Ok(())
}
