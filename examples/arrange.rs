extern crate rand;
extern crate timely;
extern crate timely_sort;
extern crate differential_dataflow;
extern crate vec_map;

use vec_map::VecMap;

use timely::dataflow::*;
use timely::dataflow::operators::*;

use timely_sort::Unsigned;

use rand::{Rng, SeedableRng, StdRng};

use differential_dataflow::AsCollection;
use differential_dataflow::operators::*;
use differential_dataflow::collection::Trace;

fn main() {

    let nodes: u32 = std::env::args().nth(1).unwrap().parse().unwrap();
    let edges: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let batch: usize = std::env::args().nth(3).unwrap().parse().unwrap();

    // define a new computational scope, in which to run BFS
    timely::execute_from_args(std::env::args().skip(5), move |computation| {

    	let index = computation.index();
    	let peers = computation.peers();

    	// create a a degree counting differential dataflow
    	let (mut input, probe, trace) = computation.scoped(|scope| {

    		// create edge input, count a few ways.
    		let (input, edges) = scope.new_input();

    		// pull off source, and count.
    		let arranged = edges.as_collection()
    							.arrange_by_key(|k: &u32| k.as_u64(), |x| (VecMap::new(), x));

		    (input, arranged.stream.probe().0, arranged.trace.clone())
    	});

        let seed: &[_] = &[1, 2, 3, index];
        let mut rng1: StdRng = SeedableRng::from_seed(seed);    // rng for edge additions
        let mut rng2: StdRng = SeedableRng::from_seed(seed);    // rng for edge additions

        // load up graph dataz
        for edge in 0..edges {
        	if edge % peers == index {
        		input.send(((rng1.gen_range(0, nodes), rng1.gen_range(0, nodes)), 1));
        	}

        	// move the data along a bit
        	if edge % 10000 == 9999 {
        		computation.step();
        	}
		}

		let timer = ::std::time::Instant::now();

		input.advance_to(1);
		computation.step_while(|| probe.lt(input.time()));

		if index == 0 {
			let timer = timer.elapsed();
			let nanos = timer.as_secs() * 1000000000 + timer.subsec_nanos() as u64;
			println!("Loading finished after {:?}", nanos);
		}

		// change graph, forever
		if batch > 0 {
			for edge in 0usize .. {
				if edge % peers == index {
	        		input.send(((rng1.gen_range(0, nodes), rng1.gen_range(0, nodes)), 1));
	        		input.send(((rng2.gen_range(0, nodes), rng2.gen_range(0, nodes)),-1));
				}

	        	if edge % batch == (batch - 1) {

	        		let timer = ::std::time::Instant::now();

	        		let next = input.epoch() + 1;
	        		input.advance_to(next);
					computation.step_while(|| probe.lt(input.time()));

					if index == 0 {
						let timer = timer.elapsed();
						let nanos = timer.as_secs() * 1000000000 + timer.subsec_nanos() as u64;
						println!("Round {} finished after {:?}", next - 1, nanos);

						let mut count = 0;
		        		let timer = ::std::time::Instant::now();
		        		let mut borrow = trace.borrow_mut();
		        		for node in 0..nodes {
		        			for _edge in borrow.get_collection(&node, input.time()) {
		        				count += 1;
		        			}
		        		}

		        		println!("count: {} in {:?}", count, timer.elapsed());
					}
	        	}
	        }
	    }

    }).unwrap();
}