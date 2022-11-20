use std::io;
use std::ffi;
use std::env;
use crate::hresult::HRESULT;

pub type IronCoreResult<T> = Result<T, IronCoreError>;

#[derive(Debug)]
pub enum IronCoreError {
    IoError(io::Error),
    NulError(ffi::NulError),
    VarError(env::VarError),
    HresultError(HRESULT),
    LibError(libloading::Error),
    InvalidExePath,
}

impl From<io::Error> for IronCoreError {
    fn from(e: io::Error) -> IronCoreError {
        IronCoreError::IoError(e)
    }
}

impl From<ffi::NulError> for IronCoreError {
    fn from(e: ffi::NulError) -> IronCoreError {
        IronCoreError::NulError(e)
    }
}

impl From<env::VarError> for IronCoreError {
    fn from(e: env::VarError) -> IronCoreError {
        IronCoreError::VarError(e)
    }
}

impl From<libloading::Error> for IronCoreError {
    fn from(e: libloading::Error) -> IronCoreError {
        IronCoreError::LibError(e)
    }
}
