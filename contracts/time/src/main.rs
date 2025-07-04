#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use alloc::vec::Vec;
use ckb_hash::new_blake2b;
use ckb_std;

use ckb_std::high_level::load_cell_lock_hash;
use ckb_std::{
    ckb_constants::Source,
    debug,
    error::SysError,
    high_level::{load_cell_type_hash, load_input, load_script, load_script_hash},
    syscalls::load_cell,
};
use molecule::prelude::Entity;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    WaitFailure,
    InvalidFd,
    OtherEndClosed,
    MaxVmsSpawned,
    MaxFdsCreated,
    // There can only be at most one input and at most one output type ID cell
    InvalidTypeIDCellNum = 20,
    // Type id does not match args
    TypeIDNotMatch,
    // Length of type id is incorrect
    ArgsLengthNotEnough,
    InvalidTypeIDLock,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            SysError::WaitFailure => Self::WaitFailure,
            SysError::InvalidFd => Self::InvalidFd,
            SysError::OtherEndClosed => Self::OtherEndClosed,
            SysError::MaxVmsSpawned => Self::MaxVmsSpawned,
            SysError::MaxFdsCreated => Self::MaxFdsCreated,
            SysError::Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

pub fn program_entry() -> i8 {
    ckb_std::debug!("This is a time script contract!");

    match load_type_id_from_script_args(0) {
        Ok(type_id) => match validate_type_id(type_id) {
            Ok(_) => match load_always_success_from_script_args(32) {
                Ok(always_success) => {
                    let lock_hash = load_cell_lock_hash(0, Source::GroupOutput).unwrap();
                    if lock_hash == always_success {
                        return 0;
                    } else {
                        return Error::InvalidTypeIDLock as i8;
                    }
                }
                Err(e) => return e as i8,
            },
            Err(e) => return e as i8,
        },
        Err(e) => return e as i8,
    }
}

fn has_type_id_cell(index: usize, source: Source) -> bool {
    let mut buf = Vec::new();
    match load_cell(&mut buf, 0, index, source) {
        Ok(_) => true,
        Err(e) => {
            // just confirm cell presence, no data needed
            if let SysError::LengthNotEnough(_) = e {
                return true;
            }
            debug!("load cell err: {:?}", e);
            false
        }
    }
}

fn locate_first_type_id_output_index() -> Result<usize, Error> {
    let current_script_hash = load_script_hash()?;

    let mut i = 0;
    loop {
        let type_hash = load_cell_type_hash(i, Source::Output)?;

        if type_hash == Some(current_script_hash) {
            break;
        }
        i += 1
    }
    Ok(i)
}

/// Time Cell可以被创建、可以被更新、但是不可以被销毁
/// Given a 32-byte type id, this function validates if
/// current transaction confronts to the type ID rules.
pub fn validate_type_id(type_id: [u8; 32]) -> Result<(), Error> {
    if has_type_id_cell(1, Source::GroupInput) {
        debug!("There can only be at most one input time cell!");
        return Err(Error::InvalidTypeIDCellNum);
    }

    if !has_type_id_cell(0, Source::GroupOutput) || has_type_id_cell(1, Source::GroupOutput) {
        debug!("There can only one output time cell!");
        return Err(Error::InvalidTypeIDCellNum);
    }

    if !has_type_id_cell(0, Source::GroupInput) {
        // We are creating a new type ID cell here. Additional checkings are needed to ensure the type ID is legit.
        let index = locate_first_type_id_output_index()?;

        // The type ID is calculated as the blake2b (with CKB's personalization) of
        // the first CellInput in current transaction, and the created output cell
        // index(in 64-bit little endian unsigned integer).
        let input = load_input(0, Source::Input)?;
        let mut hasher = new_blake2b();
        hasher.update(&input.as_slice());
        hasher.update(&index.to_le_bytes());
        let mut ret = [0; 32];
        hasher.finalize(&mut ret);

        ckb_std::debug!("ret: {:?}, type_id: {:?}", ret, type_id);

        if ret != type_id {
            debug!("Invalid type ID!");
            return Err(Error::TypeIDNotMatch);
        }
    }
    Ok(())
}

/// Loading type ID from current script args, type_id must be at least 32 byte
/// long.
pub fn load_type_id_from_script_args(offset: usize) -> Result<[u8; 32], Error> {
    let script = load_script()?;
    let args = script.as_reader().args();
    if offset + 32 > args.raw_data().len() {
        debug!("Length of type id is incorrect!");
        return Err(Error::ArgsLengthNotEnough);
    }
    let mut ret = [0; 32];
    ret.copy_from_slice(&args.raw_data()[offset..offset + 32]);
    Ok(ret)
}

/// Loading always_success from current script args, always_success must be at least 32 byte
/// long.
pub fn load_always_success_from_script_args(offset: usize) -> Result<[u8; 32], Error> {
    let script = load_script()?;
    let args = script.as_reader().args();
    if offset + 32 > args.raw_data().len() {
        debug!(
            "Length of always success is incorrect! args len is {}",
            args.raw_data().len()
        );
        return Err(Error::ArgsLengthNotEnough);
    }
    let mut ret = [0; 32];
    ret.copy_from_slice(&args.raw_data()[offset..offset + 32]);
    Ok(ret)
}
