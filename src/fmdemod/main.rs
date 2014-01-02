extern mod extra;
extern mod kissfft;
extern mod videoSinkSDL2;
extern mod rtlsdr;
extern mod pa;
extern mod dsputils;

use std::num;
use extra::complex;
use extra::time;

fn main() {
	let devHandle = rtlsdr::openDevice(0);
	rtlsdr::setSampleRate(devHandle, 881000);
	rtlsdr::clearBuffer(devHandle);
	rtlsdr::setGainAuto(devHandle);
	rtlsdr::setFrequency(devHandle, 87900000);
	let (p, videoChan) = videoSinkSDL2::spawnVectorVisualSink();
	let pi: f32 = num::atan2(1f32,1f32) * 4f32;
	let co = pa::buildPASinkBlock(44100);
	// len 511 complex-coefficiented, real-valued block filter, padded to 8192
	let filter: ~[f32] = dsputils::bpf(511, 20.0/881e3, 20e3/881e3);
	let paddedFilter: ~[f32] = ~[0.0f32, ..3840] + filter + ~[0.0f32, ..3841];
	let filterFFTd: ~[complex::Complex32] = kissfft::kissFFT(dsputils::asRe(paddedFilter));
	let pdata = rtlsdr::readAsync(devHandle, 512*15);
	loop {
		let start = time::precise_time_ns();
		let samples = rtlsdr::dataToSamples(pdata.recv());
		// phase of complex numbers
		let phase: ~[f32] = samples.iter().map(|&x| num::atan2(x.im, x.re)).collect();
		let dpdt: ~[f32] = phase.window_iter(2).map(|x| {
			let mut dx = x[0]-x[1];
			if (dx < -pi) {dx = dx + 2f32*pi};
			if (dx > pi) {dx = dx - 2f32*pi};
			dx}
			).collect::<~[f32]>();
		// 15 * 512 = 7679
		let paddedData = dsputils::asRe(~[0.0f32, ..256]) + dsputils::asRe(dpdt) + dsputils::asRe(~[0.0f32, ..257]);
		// multiply DFT'd filter coefficients by DFT'd data - implement overlap-scrap fast convolution
		// - http://www.dspguide.com/ch12/1.htm
		// - https://en.wikipedia.org/wiki/Overlap%E2%80%93add_method
		// - http://www.cs.princeton.edu/courses/archive/spr05/cos423/lectures/05fft.pdf
		let datafft = kissfft::kissFFT(paddedData);
		let convolved: ~[complex::Complex32] = datafft.iter().zip(filterFFTd.iter()).map(|(&x, &y)| {x*y}).collect();
		let filtered: ~[f32] = dsputils::asF32(kissfft::kissiFFT(convolved));
		let trimmed: ~[f32] = filtered.iter().enumerate().filter(|&(x, &y)| (256<x)  && (x<=((8192/2)-256))).map(|(x, &y)| y).collect();
		// too-clever downsampling to ensure consistent 44.1k sample rate
		let samples = (time::precise_time_ns() - start) as f32 * 44.1e-6;
		let len = trimmed.len() as f32;
		let downsampleFactor = samples/len;
		let downsampled: ~[f32] = trimmed.iter().enumerate().filter(|&(x, &y)|
			((x as f32*downsampleFactor) - (x as f32*downsampleFactor).floor()) < downsampleFactor
			).map(|(x, &y)| y).collect();
		videoChan.send(downsampled.clone());
		co.send(downsampled.clone());
		let end = time::precise_time_ns();
		println!("{} {}", (end-start)/1000, downsampled.len());
	}
}
