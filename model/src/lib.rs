#![feature(iter_repeat_n)]
use numpy::PyReadonlyArray1;
use peroxide::c;
use peroxide::fuga::*;
use pyo3::exceptions::{PyKeyError, PyRuntimeError};
use pyo3::prelude::*;
use toml::Value;

#[pyclass]
#[derive(Default, Debug, Copy, Clone)]
pub struct Pid {
    /// Numerator coefficients
    a: [f64; 3],
    /// Denominator coefficients
    b: [f64; 2],
    /// Previous inputs
    inputs: [f64; 2],
    /// Previous outputs
    outputs: [f64; 2],
    /// Evaluation period
    T: f64,
}

#[pymethods]
impl Pid {
    #[new]
    pub fn new(kp: f64, ti: f64, td: f64, n: u32, T: f64) -> Self {
        let n = f64::from(n);
        match (kp, ti, td) {
            // P
            (kp, 0.0, 0.0) => Self {
                a: [kp, 0.0, 0.0],
                b: [0.0, 0.0],
                T,
                ..Default::default()
            },
            // PID filtered
            _ => {
                // transfer function coefficient in Laplace
                let a2 = kp * td * (n + 1.0) / n;
                let a1 = kp * (td + ti * n) / (ti * n);
                let a0 = kp / ti;
                let b2 = td / n;
                // transfer function coefficient in Z
                let c2 = 4.0 * a2 + 2.0 * T * a1 + T.powi(2) * a0;
                let c1 = -8.0 * a2 + 2.0 * T.powi(2) * a0;
                let c0 = 4.0 * a2 - 2.0 * T * a1 + T.powi(2) * a0;
                let d2 = 4.0 * b2 + 2.0 * T;
                let d1 = -8.0 * b2;
                let d0 = 4.0 * b2 - 2.0 * T;

                Self {
                    a: [c2 / d2, c1 / d2, c0 / d2],
                    b: [d1 / d2, d0 / d2],
                    T,
                    ..Default::default()
                }
            },
        }
    }

    pub fn __str__(&self) -> String {
        format!("PID: a = {:?}, b = {:?}", self.a, self.b).to_string()
    }

    pub fn update(&mut self, input: f64) -> f64 {
        let output = input * self.a[0] + self.inputs[0] * self.a[1] + self.inputs[1] * self.a[2]
            - self.outputs[0] * self.b[0]
            - self.outputs[1] * self.b[1];
        self.inputs[1] = self.inputs[0];
        self.inputs[0] = input;
        self.outputs[1] = self.outputs[0];
        self.outputs[0] = output;
        self.outputs[0]
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct Config {
    size: f64,
    jx: f64,
    jy: f64,
    jz: f64,
    tm: f64,
    cr: f64,
    wb: f64,
    ct: f64,
    cm: f64,
    throttle: f64,
    w: f64,
}

#[derive(Default, Copy, Clone)]
pub struct Drone {
    config: Config,
    set_point: f64,
    throttles: [f64; 4],
}

impl Environment for Drone {}

#[pyclass]
struct Model {
    config: Config,
    set_point: f64,
    thrust: Option<Vec<f64>>,
}

#[pymethods]
impl Model {
    #[new]
    fn new(path: String, set_point: f64, thrust: Option<PyReadonlyArray1<f64>>) -> PyResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Value = toml::from_str(&content).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let config = Config {
            size: config["frame"]["size"].as_float().ok_or(PyKeyError::new_err("frame/size"))?,
            jx: config["frame"]["jx"].as_float().ok_or(PyKeyError::new_err("frame/jx"))?,
            jy: config["frame"]["jy"].as_float().ok_or(PyKeyError::new_err("frame/jy"))?,
            jz: config["frame"]["jz"].as_float().ok_or(PyKeyError::new_err("frame/jz"))?,
            tm: config["motor"]["tm"].as_float().ok_or(PyKeyError::new_err("motor/tm"))?,
            cr: config["motor"]["cr"].as_float().ok_or(PyKeyError::new_err("motor/cr"))?,
            wb: config["motor"]["wb"].as_float().ok_or(PyKeyError::new_err("motor/wb"))?,
            ct: config["propeller"]["ct"].as_float().ok_or(PyKeyError::new_err("propeller/ct"))?,
            cm: config["propeller"]["cm"].as_float().ok_or(PyKeyError::new_err("propeller/cm"))?,
            throttle: config["hover"]["throttle"].as_float().ok_or(PyKeyError::new_err("hover/throttle"))?,
            w: config["hover"]["w"].as_float().ok_or(PyKeyError::new_err("hover/w"))?,
        };

        let thrust = thrust.map(|x| x.as_slice().unwrap().to_vec());

        Ok(Self {
            config,
            set_point,
            thrust,
        })
    }

    #[pyo3(signature = (pid, save=false))]
    fn __call__(&self, pid: PyReadonlyArray1<f64>, save: bool) -> f64 {
        let kp = *pid.get(0).unwrap_or(&0.0);
        let ti = *pid.get(1).unwrap_or(&0.0);
        let td = *pid.get(2).unwrap_or(&0.0);
        let kpp = *pid.get(3).unwrap_or(&1.0);

        let set_point = self.set_point;

        let mut pid_velocity = Pid::new(kp, ti, td, 5, 0.01);
        let mut pid_position = Pid::new(kpp, 0.0, 0.0, 5, 0.01);

        let mut drone = Drone {
            config: self.config,
            set_point,
            throttles: [self.config.throttle; 4],
        };

        let w = if let Some(thrust) = self.thrust.as_ref() {
            if thrust[0] == 0.0 {
                0.0
            } else {
                drone.config.cr * thrust[0] + drone.config.wb
            }
        } else {
            drone.config.w
        };
        let state = State::<f64>::new(0f64, c!(w, w, w, w, 0, 0, 0, 0, 0, 0), c!(0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

        let mut ode_solver = ExplicitODE::new(compute_accel);

        ode_solver
            .set_method(ExMethod::RK4)
            .set_initial_condition(state)
            .set_env(drone)
            .set_step_size(0.001)
            .set_times(10);

        let default_thrust = if self.thrust.is_some() {
            std::iter::repeat(&drone.config.throttle).take(0)
        } else {
            std::iter::repeat(&drone.config.throttle).take(100)
        };

        let mut errors = vec![(0.0, 0.0)];
        let mut record = Vec::new();
        for _ in self.thrust.as_ref().unwrap_or(&vec![]).iter().chain(default_thrust) {
            let mut result = ode_solver.integrate().row(10);
            if ode_solver.has_stopped() {
                return f64::MAX;
            }
            let time = result[0];
            let _motors: [_; 4] = result[1..=4].try_into().unwrap();
            let [vroll, _vpitch, _vyaw] = result[5..=7].try_into().unwrap();
            let [roll, _pitch, _yaw] = result[8..=10].try_into().unwrap();

            // PID computation
            let cmd_roll = pid_position.update(set_point - roll);
            let cmd_vroll = if kpp != 0.0 {
                errors.push((time, set_point - roll));
                pid_velocity.update(cmd_roll - vroll)
            } else {
                errors.push((time, set_point - vroll));
                pid_velocity.update(set_point - vroll)
            };

            // Motor allocation with PWM consideration
            // The output of PID is truncated to simulate cast to int
            // The output of PID is divided by 200000 to be between [0:1] instead of[0:200000]
            drone.throttles = [
                drone.config.throttle + (cmd_vroll.trunc() / 200_000.0),
                drone.config.throttle + (-cmd_vroll.trunc() / 200_000.0),
                drone.config.throttle + (-cmd_vroll.trunc() / 200_000.0),
                drone.config.throttle + (cmd_vroll.trunc() / 200_000.0),
            ];

            for throttle in drone.throttles {
                if !(0.0..=200_000.0).contains(&throttle) {
                    return f64::MAX;
                }
            }

            ode_solver.set_env(drone);
            if save {
                result.push(cmd_vroll);
                result.push(cmd_roll);
                record.push(result);
            }
        }

        if save {
            dump_csv(record);
        }

        errors.iter().map(|(t, e)| e.abs() * t).sum::<f64>()
    }
}
/**
 * 0 Motor 0 | 4 Wx | 7 Px
 * 1 Motor 1 | 5 Wy | 8 Py
 * 2 Motor 2 | 6 Wz | 9 Pz
 * 3 Motor 3 |
 */
pub fn compute_accel(state: &mut State<f64>, env: &Drone) {
    for i in 0..4 {
        state.deriv[i] =
            (env.config.cr * env.throttles[i].clamp(0.0, 1.0) + env.config.wb - state.value[i]) / env.config.tm;
    }

    let w0 = state.value[0].powi(2);
    let w1 = state.value[1].powi(2);
    let w2 = state.value[2].powi(2);
    let w3 = state.value[3].powi(2);
    let d = (2.0.sqrt() / 2.0) * env.config.size;
    // Wx
    state.deriv[4] = d * env.config.ct * (w0 - w1 - w2 + w3) / env.config.jx;
    // Wy
    state.deriv[5] = d * env.config.ct * (w0 + w1 - w2 - w3) / env.config.jy;
    // Wz
    state.deriv[6] = env.config.cm * (w0 - w1 + w2 - w3) / env.config.jz;

    // Px
    state.deriv[7] = state.value[4];
    // Py
    state.deriv[8] = state.value[5];
    // Pz
    state.deriv[9] = state.value[6];
}

#[pymodule]
fn model(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pid>()?;
    m.add_class::<Model>()?;
    Ok(())
}

fn dump_csv(mut record: Vec<Vec<f64>>) {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create("result.csv").unwrap();

    for line in record.iter_mut() {
        let last = line.pop().unwrap();
        for value in line {
            write!(&mut file, "{},", value).unwrap();
        }
        write!(&mut file, "{}\n", last).unwrap();
    }
}
