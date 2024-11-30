use crate::types::{Angles, Command, FlightCommand};
use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::ffi::CString;
use std::path::Path;
use std::sync::mpsc::Sender;

#[pyclass(name = "Command")]
#[derive(Debug, Clone)]
enum CommandKind {
    Raw(u32, u32, u32, u32),
    Position(f32, f32, f32, f32),
    Velocity(f32, f32, f32, f32),
}

#[pyclass(frozen)]
struct Comm {
    pub tx: Sender<Command>,
}

#[pyclass(subclass)]
struct Plugin {
    comm: Py<Comm>,
}

#[pymethods]
impl Plugin {
    #[new]
    fn new(comm: Py<Comm>) -> Self {
        Self {
            comm,
        }
    }

    fn start(&self) {
        log::warn!("You shall implement start as your plugin entry point!");
    }

    fn log(&self, msg: String) {
        log::info!("{}", msg);
    }

    fn send(&self, cmd: CommandKind) -> PyResult<()> {
        let comm = self.comm.get();
        match cmd {
            CommandKind::Raw(m0, m1, m2, m3) => {
                comm.tx
                    .send(Command::SetMotor {
                        motor: 0,
                        value: m0,
                    })
                    .unwrap();
                comm.tx
                    .send(Command::SetMotor {
                        motor: 1,
                        value: m1,
                    })
                    .unwrap();
                comm.tx
                    .send(Command::SetMotor {
                        motor: 2,
                        value: m2,
                    })
                    .unwrap();
                comm.tx
                    .send(Command::SetMotor {
                        motor: 3,
                        value: m3,
                    })
                    .unwrap();
            },
            CommandKind::Position(thrust, roll, pitch, yaw) => {
                comm.tx
                    .send(Command::Flight(FlightCommand {
                        thrust,
                        angles: Angles {
                            roll,
                            pitch,
                            yaw,
                        },
                    }))
                    .unwrap();
            },
            _ => unimplemented!(),
        };
        Ok(())
    }
}

#[pyfunction]
#[pyo3(pass_module)]
fn register<'py>(module: &Bound<'py, PyModule>, cls: Bound<'py, PyType>) {
    module.setattr(pyo3::intern!(module.py(), "entry"), cls).unwrap();
}

#[pymodule(name = "drosix")]
fn pymodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CommandKind>()?;
    m.add_class::<Comm>()?;
    m.add_class::<Plugin>()?;
    m.add_function(wrap_pyfunction!(register, m).unwrap())
}

pub fn run_plugin<P: AsRef<Path>>(path: P, command_tx: Sender<Command>) -> Result<()> {
    log::info!("Starting plugin {}", path.as_ref().display());

    pyo3::append_to_inittab!(pymodule);
    pyo3::prepare_freethreaded_python();
    let code = std::fs::read_to_string("plugin.py").expect("You should provide valid python plugin path");
    Python::with_gil(|py| {
        PyModule::from_code(
            py,
            CString::new(code).unwrap().as_c_str(),
            pyo3::ffi::c_str!("plugin.py"),
            pyo3::ffi::c_str!("plugin"),
        )
        .inspect_err(|e| log::error!("{}", e))?;
        py.import("drosix")
            .and_then(|module| module.getattr(pyo3::intern!(py, "entry")))
            .and_then(|entry| {
                entry.call1((Comm {
                    tx: command_tx,
                },))
            })
            .and_then(|plugin| plugin.getattr("start"))
            .and_then(move |start| start.call0())
            .map(|_| log::info!("Plugin {} finished", path.as_ref().display()))
            .map_err(|e| anyhow::anyhow!(e))
            .inspect_err(|e| log::error!("{}", e))
    })
}
