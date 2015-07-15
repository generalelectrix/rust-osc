extern crate time;

/// THIS CODE HAS NOT BEEN PORTED TO MODERN (1.3.0) RUST YET.

/// Create and run a new broadcaster, returning the sender used to broadcast a
/// message to multiple consumers, and the control structure used to generate a
/// new receiver for a consumer.
///
/// # Example
///
/// ```
/// let (bc, bc_ctrl) = broadcaster();
///
/// // Spawn some children that you would like to broadcast to:
/// for i in range(0,10) {
/// 	let rx = bc_ctrl.tune_in();
/// 	spawn(proc(){
/// 		loop {
/// 			println!("task {} received {}", i, rx.recv());
/// 		}
/// 	});
/// }
///
/// // Send a message to all of them
/// bc.send("hello, my children");
pub fn broadcaster<T: Send + Clone>() -> (Sender<T>, BcCtrl<T>) {

	// create the channels to send messages using this bc as well as to give it new consumers
	let (tx_bc, receiver) = channel();
	let (tx_ctrl, ctrl_chan) = channel();

	// spin up a task the BC will run in
	spawn(proc() {

		let mut senders: Vec<Sender<T>> = Vec::new();

		// keep track of whether or not the controller still exists
		let mut ctrl_alive = true;

		// transcieve!
		loop {

			let to_send: T;


			// block for a message to send, or kill the task if the sender has hung up
			match receiver.recv_opt() {
				Ok(v) => to_send = v,
				Err(_) => break
			}

			// check to see if we need to add any new consumers
			if ctrl_alive {
				loop {
					match ctrl_chan.try_recv() {
						Ok(c) => senders.push(c),
						// for now, do not kill the BC if the controller hangs up!
						// there may be use cases where we eliminate the controller
						// to prevent adding more consumers
						Err(e) if e == std::comm::Empty => break,
						Err(_) => {
							ctrl_alive = false;
							break
						}
					}
				}

			}


			// here in one step we broadcast and also remove channels which have hung up
			senders.retain(|chan|
				match chan.send_opt(to_send.clone()) {
					Ok(_) => true,
					Err(_) => false
				}
			);
		}

	});

	(tx_bc, BcCtrl{ctrl_chan: tx_ctrl})
}

/// Helper object to request a new receiver from the broadcaster.
pub struct BcCtrl<T>{
	ctrl_chan: Sender<Sender<T>>
}


impl<T: Send + Clone> BcCtrl<T> {

	/// Create a new receiver from the Broadcaster under control.
	pub fn tune_in(&self) -> Receiver<T> {
		let (tx, rx) = channel();
		self.ctrl_chan.send(tx);

		rx
	}
}

fn main() {

	let n = 1000u64;

	let (tx, ctrl) = broadcaster();

	let t1 = time::precise_time_ns();
	let t2 = time::precise_time_ns();
	println!("{}", t2 - t1);

	let mut bigvec = Vec::new();
	for i in range(0,100) {
		bigvec.push(i);
	}

	spawn(proc(){
		for _ in range(0,n) {

			let tbvec = bigvec.clone();
			std::io::timer::sleep(10);

			tx.send( (time::precise_time_ns(), tbvec) );
		}
	});

	for _ in range(0,10) {

		let rx = ctrl.tune_in();

		spawn(proc(){

			let mut ave = 0;

			for _ in range(0,n) {
				let (t_sent, bvec) = rx.recv();
				let t_now = time::precise_time_ns();
				ave += t_now - t_sent;
				bvec.len();
			}

			println!("{} ns", ave / n)

		});
	}

	println!("Dropping broadcaster control structure.")
	drop(ctrl);

}
