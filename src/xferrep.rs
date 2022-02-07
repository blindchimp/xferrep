use std::result::Result;
use std;
// super simple version of xfer-rep parser
// ref: serde, but this is just a learning project, so
// avoiding it for now.

pub type XferErr = i32;

// note: the api for this stream may not be rust friendly
// as it is doing something that is more C-like: "give me a pointer
// to a buffer, and i'll fill it in (i promise not to mess it up, honest)"
// BUT, just from a learning perspective, having to specify the lifetimes
// is interesting.

pub trait XStream {
	fn out_want<'a>(&mut self, s: usize) -> Result<&'a mut [u8], XferErr>;
	fn in_want<'a>(&self, s: usize) -> Result<&'a [u8], XferErr>;
}

trait XferRep {
	fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> ;
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> ;
}

pub enum Vc {
	VcNil,
	VcInt {i: i64},
}

impl XferRep for () {
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
	let p = x.out_want(2).unwrap();
	p[0] = b'0';
	p[1] = b'4';
	Result::Ok(2)
	}

	fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> {
	Result::Err(-1)
	}
}

impl XferRep for i64 {
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
	let p = x.out_want(2).unwrap();
	p[0] = b'0';
	p[1] = b'1';

	let snum = format!("{}", self);
	let bnum = snum.as_bytes();
	let blen = bnum.len();
	if blen > 99 {
		return Result::Err(-1);
	}

	let p = x.out_want(blen + 2).unwrap();
	p[0] = (blen / 10) as u8 + b'0';
	p[1] = (blen % 10) as u8 + b'0';
	let mut i = 0;
	while i < blen {
		p[i + 2] = bnum[i];
		i += 1;
	}
	Result::Ok(blen + 2)
	}

	fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> {
	Result::Err(-1)
	}
}

impl Vc {
	pub fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
	match self {
	Vc::VcNil => ().xfer_out(x),
	Vc::VcInt {i} => i.xfer_out(x),
	}
	//Result::Err(-1)
	}

	pub fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> {
	Result::Err(-1)
	}
}
