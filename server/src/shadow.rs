// Taken from https://github.com/ids1024/shadow-rs

// Copyright (C) 2017 Ian Douglas Scott
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

use libc::c_long;
use std::ffi::CStr;
use std::ffi::CString;

/// Represents an entry in `/etc/shadow`
#[derive(Debug)]
pub struct Shadow {
    /// user login name
    pub name: String,
    /// encrypted password
    pub password: String,
    /// last password change
    pub last_change: c_long,
    /// days until change allowed
    pub min: c_long,
    /// days before change required
    pub max: c_long,
    /// days warning for expiration
    pub warn: c_long,
    /// days before account inactive
    pub inactive: c_long,
    /// date when account expires
    pub expire: c_long,
}

impl Shadow {
    unsafe fn from_ptr(spwd: *const libc::spwd) -> Shadow {
        Shadow {
            name: CStr::from_ptr((*spwd).sp_namp).to_str().unwrap().to_owned(),
            password: CStr::from_ptr((*spwd).sp_pwdp)
                .to_str()
                .unwrap()
                .to_owned(),
            last_change: (*spwd).sp_lstchg,
            min: (*spwd).sp_min,
            max: (*spwd).sp_max,
            warn: (*spwd).sp_warn,
            inactive: (*spwd).sp_inact,
            expire: (*spwd).sp_expire,
        }
    }

    /// Gets a `Shadow` entry for the given username, or returns `None`
    pub fn from_name(user: &str) -> Option<Shadow> {
        let c_user = CString::new(user).unwrap();

        let spwd = unsafe { libc::getspnam(c_user.as_ptr()) };

        if spwd.is_null() {
            None
        } else {
            Some(unsafe { Shadow::from_ptr(spwd) })
        }
    }

    /// Returns iterator over all entries in `shadow` file
    pub fn iter_all() -> ShadowIter {
        ShadowIter::default()
    }
}

/// Iterator over `Shadow` entries
#[derive(Default)]
pub struct ShadowIter {
    started: bool,
    done: bool,
}

impl Iterator for ShadowIter {
    type Item = Shadow;

    fn next(&mut self) -> Option<Shadow> {
        self.started = true;
        if !self.done {
            let spwd = unsafe { libc::getspent() };
            if spwd.is_null() {
                unsafe { libc::endspent() };
                self.done = true;
                None
            } else {
                Some(unsafe { Shadow::from_ptr(spwd) })
            }
        } else {
            None
        }
    }
}

impl Drop for ShadowIter {
    fn drop(&mut self) {
        if self.started && !self.done {
            unsafe { libc::endspent() };
        }
    }
}
