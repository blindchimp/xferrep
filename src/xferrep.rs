use ascii::{AsciiString, ToAsciiChar};
use bytes::{Buf, Bytes};
use std::result::Result;

#[derive(Debug)]
pub enum Vc {
    VcNil,
    VcInt { i: i64 },
    VcStr { s: Vec<u8> },
    VcVec { vec: Vec<Vc> },
}

pub const VCNIL: Vc = Vc::VcNil;
pub const VCINTZERO: Vc = Vc::VcInt { i: 0 };
pub const VCVECEMPTY: Vc = Vc::VcVec { vec: Vec::new() };
pub const VCSTREMPTY: Vc = Vc::VcStr { s: Vec::new() };

// we know what this is, right? hard WINK.
const ZERO:u8 = 48u8;

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
    fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr>;
    fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr>;
}

impl XferRep for () {
    fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
        let p = x.out_want(2).unwrap();
        Vc::type_out(&VCNIL, p);
        Result::Ok(2)
    }

    fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
        Result::Ok(Vc::VcNil)
    }
}

impl XferRep for i64 {
    fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
        let p = x.out_want(2).unwrap();
        Vc::type_out(&VCINTZERO, p);

        let snum = format!("{}", self);
        let bnum = snum.as_bytes();
        let blen = bnum.len();
        if blen > 99 {
            return Result::Err(-1);
        }

        let p = x.out_want(blen + 2).unwrap();
        p[0] = (blen / 10) as u8 + ZERO;
        p[1] = (blen % 10) as u8 + ZERO;
        let mut i: usize = 0;
        while i < blen {
            p[i + 2] = bnum[i];
            i += 1;
        }
        Result::Ok(blen + 2)
    }

    fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
        let lenab = x.in_want(2).unwrap();
        let lena = lenab.chunk();
        let lennum = (lena[0] - ZERO) * 10 + (lena[1] - ZERO);

        let numab = x.in_want(lennum as usize).unwrap();
        let numa = numab.chunk();
        // note: this isn't right, but we'll do it just for expediency
        // right now. parse accepts more formats than we produce, which
        // is a bug.
        let num = std::str::from_utf8(numa).unwrap().parse::<i64>().unwrap();
        Result::Ok(Vc::VcInt { i: num })
    }
}

fn encode_long(num: u64) -> AsciiString {
    let snum = format!("{}", num);
    let anum = AsciiString::from_ascii(snum).unwrap();

    let lenlen = anum.len();

    let mut ret = AsciiString::new();
    ret.push(((lenlen / 10) as u8 + ZERO).to_ascii_char().unwrap());
    ret.push(((lenlen % 10) as u8 + ZERO).to_ascii_char().unwrap());

    let lenstr = AsciiString::from_ascii(format!("{}", lenlen)).unwrap();
    ret.push_str(&lenstr);
    ret.push_str(&anum);

    ret
}

// note: this isn't right, but we'll do it just for expediency
// right now. parse accepts more formats than we produce, which
// is a bug. and the input is not utf8
fn decode_long(b: &[u8]) -> u64 {
    let num = std::str::from_utf8(b).unwrap().parse::<u64>().unwrap();
    num
}

impl XferRep for Vec<u8> {
    fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
        let res = encode_long(self.len() as u64);
        let ret = res.as_bytes();
        let outstr = self.as_slice();

        let buf = x.out_want(2 + ret.len() + outstr.len()).unwrap();
        Vc::type_out(&VCSTREMPTY, buf);
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
    fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
        let tp = x.in_want(2).unwrap();
        let tpc = tp.chunk();
        let lenlen = (tpc[0] - ZERO) * 10 + (tpc[1] - ZERO);

        let lenbuf = x.in_want(lenlen as usize).unwrap();
        let strlen = decode_long(lenbuf.chunk());

        let ss = x.in_want(strlen as usize).unwrap();

        Result::Ok(Vc::VcStr { s: ss.to_vec() })


        //Result::Err(-1)
    }
}

impl XferRep for Vec<Vc> {
    fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
        let buf = x.out_want(2).unwrap();
        Vc::type_out(&VCVECEMPTY, buf);

        // number of elems in the vector
        let res = encode_long(self.len() as u64);
        let ret = res.as_bytes();

        let buf2 = x.out_want(ret.len()).unwrap();
        buf2.copy_from_slice(ret);

        let mut i = 0;
        let mut tot = 2 + ret.len();
        
        while i < self.len() {
            tot += self.get(i).unwrap().xfer_out(x).unwrap();
            i += 1;
        }
        Result::Ok(tot)
    }
    fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
        Result::Err(-1)
    }
}

impl Vc {
    pub fn xfer_out(&self, x: &mut dyn XStream) -> Result<usize, XferErr> {
        match self {
            Vc::VcNil => ().xfer_out(x),
            Vc::VcInt { i } => i.xfer_out(x),
            Vc::VcStr { s } => s.xfer_out(x),
            Vc::VcVec { vec } => vec.xfer_out(x),
        }
        //Result::Err(-1)
    }

    // note: this api in c++ returns usize, but really the amount of data
    // consumed is never used (anywhere i can see in old code). rather it is just
    // used as an error indicator. since i don't think rust is going to
    // let us modify the type of "self" as we sorta do under the covers
    // in c++, we'll just returns the enum value instead of the size.
    pub fn xfer_in(&self, x: &mut dyn XStream) -> Result<Vc, XferErr> {
        let tp = x.in_want(2).unwrap();
        let tpc = tp.chunk();
        let tpi = (tpc[0] - ZERO) * 10 + (tpc[1] - ZERO);
        match tpi {
            4 => return Result::Ok(Vc::VcNil),
            1 => {
                let r: i64 = 0;
                return Result::Ok(r.xfer_in(x).unwrap());
            },
            2 => {
                let r: Vec<u8> = Vec::new();
                return Result::Ok(r.xfer_in(x).unwrap());
            },
            9 => {

            },
            _ => (),
        };

        Result::Err(-1)
    }

    fn type_out(&self, b: &mut [u8]) -> () {
        match self {
            Vc::VcNil => {
                b[0] = ZERO;
                b[1] = ZERO + 4;
            }
            Vc::VcInt { .. } => {
                b[0] = ZERO;
                b[1] = ZERO + 1;
            }
            Vc::VcStr { .. } => {
                b[0] = ZERO;
                b[1] = ZERO + 2;
            }
            Vc::VcVec { .. } => {
                b[0] = ZERO;
                b[1] = ZERO + 9;
            }
            //_ => (),
        }
    }
}
