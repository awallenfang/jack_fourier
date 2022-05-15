use ringbuf::Producer;
use jack::{self, AsyncClient, Client, ClosureProcessHandler, Control, ProcessScope};

pub fn start_jack(mut producer: Producer<f32>) -> AsyncClient<(), ClosureProcessHandler<fn(&Client, &ProcessScope) -> Control>> {
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
                producer.push(*input_l);
            }

            // Continue as normal
            jack::Control::Continue
        }
    );

    client.activate_async((), process).unwrap()
}