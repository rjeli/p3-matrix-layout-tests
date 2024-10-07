use std::{cmp, mem, time::Instant};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

fn main() {
    println!("testing metal");

    let mut rng = ChaChaRng::seed_from_u64(0);

    let log_n = 26;
    let n = 1 << log_n;

    objc::rc::autoreleasepool(|| {
        let device = metal::Device::system_default().expect("no device found");

        let library = device
            .new_library_with_data(include_bytes!("../shader.metallib"))
            .unwrap();
        let kernel = library.get_function("sum", None).unwrap();

        println!("Function name: {}", kernel.name());
        println!("Function type: {:?}", kernel.function_type());
        println!("OK");

        //        let desc = metal::ComputePipelineDescriptor::new();
        //        desc.set_compute_function(Some(&kernel));

        let pipeline_state = device
            .new_compute_pipeline_state_with_function(&kernel)
            .unwrap();

        let buf_sz = mem::size_of::<u32>() << log_n;
        let buf_out =
            device.new_buffer(buf_sz as u64, metal::MTLResourceOptions::StorageModeShared);
        let buf_a = device.new_buffer(buf_sz as u64, metal::MTLResourceOptions::StorageModeShared);
        let buf_b = device.new_buffer(buf_sz as u64, metal::MTLResourceOptions::StorageModeShared);

        let buf_out_ptr = buf_out.contents() as *mut u32;
        let buf_a_ptr = buf_a.contents() as *mut u32;
        let buf_b_ptr = buf_b.contents() as *mut u32;

        unsafe {
            for i in 0..n {
                *buf_a_ptr.add(i) = rng.gen();
                *buf_b_ptr.add(i) = rng.gen();
            }
        }

        let cq = device.new_command_queue();

        for _ in 0..10 {
            let cbuf = cq.new_command_buffer();

            let cenc = cbuf.new_compute_command_encoder();
            cenc.set_compute_pipeline_state(&pipeline_state);

            cenc.set_buffer(0, Some(&buf_out), 0);
            cenc.set_buffer(1, Some(&buf_a), 0);
            cenc.set_buffer(2, Some(&buf_b), 0);

            cenc.dispatch_threads(
                // threads per grid
                metal::MTLSize::new(n as u64, 1, 1),
                // threads per threadgroup
                metal::MTLSize::new(
                    cmp::min(n as u64, pipeline_state.max_total_threads_per_threadgroup()),
                    1,
                    1,
                ),
            );
            cenc.end_encoding();

            println!("committing");
            let t0 = Instant::now();
            cbuf.commit();

            println!("waiting");
            cbuf.wait_until_completed();
            let t1 = Instant::now();

            let dt = t1 - t0;

            let bw = ((3 * buf_sz) as f32) / dt.as_secs_f32() / 1024.0 / 1024.0 / 1024.0;

            println!("done in {dt:?} ({bw} GB/s)");

            /*
            unsafe {
                for i in 0..n {
                    let a = *buf_a_ptr.add(i);
                    let b = *buf_b_ptr.add(i);
                    let out = *buf_out_ptr.add(i);
                    assert_eq!(a + b, out, "at {i}: {a} + {b} != {out}");
                }
            }
            */
        }

        // verify

        // cenc.dispatch_thread_groups(thread_groups_count, threads_per_threadgroup);

        // let p = device.new_compute_pipeline_state(descriptor)

        // b.counter(BytesCount::new(1usize)).bench_local(|| {});
    });
}
