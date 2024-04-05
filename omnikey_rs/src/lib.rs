//! An open source wrapper package for the
//! OMNIKEY 5025CL.
//! 
//! # Overview
//! 
//! This package contains two userspace structs
//! for setting up, as well as reading from and
//! writing to, the OMNIKEY 5025CL. We use the
//! CCID spec
//! [linked here](https://www.usb.org/sites/default/files/DWG_Smart-Card_CCID_Rev110.pdf),
//! as well as the OMNIKEY Developer Guide
//! [here](https://www.hidglobal.com/documents/omnikey-5025-cl-software-development-guide)
//! to develop the API. The backend relies on
//! libUSB.
//! 
//! Important to note is that at current, this
//! crate relies on the host system being
//! little endian. This works for most modern
//! systems, but for older/more niche systems,
//! this crate may not work!

pub mod structs;
