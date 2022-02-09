use std::result::Result;
use ascii::{AsciiString, ToAsciiChar};
use bytes::{Bytes, Buf};


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
	fn in_want(&mut self, s: usize) -> Result<Bytes, XferErr>;
}

trait XferRep {
	fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> ;
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> ;
}

pub enum Vc {
	VcNil,
	VcInt {i: i64},
	VcStr {s: Vec<u8>},
	VcVec {vec: Vec<Vc>},
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
	let mut i: usize = 0;
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

fn encode_long(num: u64) -> AsciiString {
	let snum = format!("{}", num);
	let anum = AsciiString::from_ascii(snum).unwrap();

	let lenlen = anum.len();

	let mut ret = AsciiString::new();
	ret.push(((lenlen / 10) as u8 + b'0').to_ascii_char().unwrap());
	ret.push(((lenlen % 10) as u8 + b'0').to_ascii_char().unwrap());

	let lenstr = AsciiString::from_ascii(format!("{}", lenlen)).unwrap();
	ret.push_str(&lenstr);
	ret.push_str(&anum);

	ret


}

impl XferRep for Vec<u8> {
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
		let res = encode_long(self.len() as u64);
		let ret = res.as_bytes();
		let outstr = self.as_slice();

		let mut buf = x.out_want(2 + ret.len() + outstr.len()).unwrap();
		buf[0] = b'0';
		buf[1] = b'2';
		let mut i = 0;
		while i < ret.len() {
			buf[i + 2] = ret[i];
			i += 1;
		}
		i = 0;
		while i < outstr.len() {
			buf[2 + ret.len() + i] = outstr[i];
			i += 1;
		}

		Result::Ok(buf.len())
	}
	fn xfer_in(&self, x: & dyn XStream) -> Result<usize, XferErr> {
		Result::Err(-1)
		}
}

impl XferRep for Vec<Vc> {
	fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
		let b = x.out_want(3).unwrap();
		b[0] = b'v';
		b[1] = b'e';
		b[2] = b'c';
		let mut i = 0;
		let mut tot = 3;
		while i < self.len() {
			tot += self.get(i).unwrap().xfer_out(x).unwrap();
			i += 1;
		}
		Result::Ok(tot)
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
	Vc::VcStr {s} => s.xfer_out(x),
	Vc::VcVec {vec} => vec.xfer_out(x),
	}
	//Result::Err(-1)
	}

	// note: this api returns usize, but really the amount of data
	// consumed is never used (anywhere i can see). rather it is just
	// used as an error indicator. since i don't think rust is going to
	// let us modify the type of "self" as we sorta do under the covers
	// in c++, we'll just returns the enum value instead of the size.
	pub fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
		let tp = x.in_want(2).unwrap();
		let tpc = tp.chunk();
		let tpi = (tpc[0] - b'0') * 10 + (tpc[1] - b'0');
		match tpi {
		4 => return(Result::Ok(Vc::VcNil)),
		_ => (),
		};

	Result::Err(-1)
	}
}
