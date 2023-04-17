use rand::{seq::SliceRandom, Rng};
use rodio::*;
use std::{io, time::Duration};
use termion::{event::Key, input::TermRead};
use csv::WriterBuilder;
use chrono::prelude::*;

// Helper function to generate frequencies based on piano notes
fn piano_freq(note: u32) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

// Helper function to create a sound wave
fn create_sound_wave(frequency: f32, amplitude: f32) -> impl rodio::Source<Item = f32> {
    rodio::source::SineWave::new(frequency).amplify(amplitude).take_duration(Duration::from_secs_f32(1.0))
}

// Helper function to play a sound wave
fn play_sound(frequency: f32, amplitude: f32, ear: &str) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::SpatialSink::try_new(&handle, [-2.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]).unwrap();

    if ear == "right" {
        sink.set_emitter_position([2.0, 0.0, 0.0]);
    } else {
        sink.set_emitter_position([-2.0, 0.0, 0.0]);
    }

    let sound_wave = create_sound_wave(frequency, amplitude);
    sink.append(sound_wave);
    sink.sleep_until_end();
}

// Helper function to print instructions
fn print_instructions() {
    println!("Welcome to the diplacusis hearing test!");
    println!("Instructions:");
    println!("  - w/d keys to change the right ear frequency.");
    println!("  - + and - keys to change the volume.");
    println!("  - Space key to replay the frequencies.");
    println!("  - # key to lock in frequencies.");
    println!("The test will end when enough coverage of the frequency band is reached.");
}

// Helper function to append results to a CSV file
fn append_results_to_csv(left_note: u32, right_note: u32) {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let file_name = format!("results_{}.csv", date);

    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_path(&file_name)
        .unwrap_or_else(|_| WriterBuilder::new().has_headers(true).from_path(&file_name).unwrap());

    writer.write_record(&[left_note.to_string(), right_note.to_string()]).unwrap();
    writer.flush().unwrap();
}

fn main() {
    let note_min = 51;
    let note_max = 108;
    let notes: Vec<u32> = (note_min..=note_max).step_by(5).collect();
    println!("Notes: {:?}", notes);

    let mut rng = rand::thread_rng();

    let mut results = Vec::new();

    print_instructions();

    let mut volume = 0.1;
    let mut key = Key::Char(' ');
    for left_note in notes.iter() {
        if key == Key::Char('q') {
            break;
        }
        // Determine the right note as a random value between the left note - 5 and the left note + 5
        let mut right_note = *left_note as i32 + rng.gen_range(-5..=5);
        let mut right_note = std::cmp::max(note_min as i32, std::cmp::min(right_note, note_max as i32)) as u32;
        
        // Lookup the frequency for the left and right notes
        let left_freq = piano_freq(*left_note);
        let mut right_freq = piano_freq(right_note);

        // Play the sound waves?
        let mut locked = false;

        while !locked {
            play_sound(left_freq, volume, "left");
            play_sound(right_freq, volume, "right");


            key = io::stdin().keys().next().unwrap().unwrap();

            match key {
                Key::Char('w') => {
                    right_note = std::cmp::min(right_note + 1, note_max);
                    right_freq = piano_freq(right_note);
                }
                Key::Char('d') => {
                    right_note = std::cmp::max(right_note - 1, note_min);
                    right_freq = piano_freq(right_note);
                }
                Key::Char('+') => {
                    volume = (volume * 10.0 + 1.0) / 10.0;
                }
                Key::Char('-') => {
                    volume = (volume * 10.0 - 1.0) / 10.0;
                }
                Key::Char('#') => {
                    println!("Locked frequencies: Left: {:.2} Hz, Right: {:.2} Hz", left_freq, right_freq);
                    results.push((left_note, right_note));
                    append_results_to_csv(*left_note, right_note);
                    locked = true;
                }
                Key::Char(' ') => {
                }
                Key::Char('q') => {
                    println!("Test aborted.");
                    return;
                }
                _ => {}
            }
        }

        if results.len() == notes.len() {
            break;
        }
    }

    println!("Test completed. Results saved to the corresponding results_YYYY-MM-DD.csv file.");
}