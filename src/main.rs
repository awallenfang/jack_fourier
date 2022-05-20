use std::sync::{Mutex, Arc};

use ringbuf::RingBuffer;
use jack;

mod dsp;
mod ui;

pub const BUFFER_SIZE: usize = 256;
pub const FFT_SIZE: usize = 4096;

fn main() {
    let jack_dsp_rb = RingBuffer::<f32>::new(50_000);
    let (mut jack_dsp_prod, jack_dsp_cons) = jack_dsp_rb.split();
    
    let (client, _status) =
        jack::Client::new("jack_fourier", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_port = client
        .register_port("fourier_in", jack::AudioIn::default())
        .unwrap();

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let in_p = in_port.as_slice(ps);

            // Write output
            for input_l in in_p {
                // Check if the current volume is at the destination by checking if there's steps left
                jack_dsp_prod.push(*input_l);
            }

            // Continue as normal
            jack::Control::Continue
        }
    );

    let sr = client.sample_rate();

    let jack_client = client.activate_async((), process).unwrap();

    let dsp_ui_mutex = Arc::new(Mutex::new(vec![-90.;1024]));

    dsp::process_thread(jack_dsp_cons, dsp_ui_mutex.clone());
    
    ui::ui(dsp_ui_mutex.clone(), sr);
}
