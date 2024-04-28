use numpy::PyReadonlyArray1;
use peroxide::c;
use peroxide::fuga::*;
use pyo3::exceptions::{PyKeyError, PyRuntimeError};
use pyo3::prelude::*;
use std::cell::RefCell;
use toml::Value;

#[pyclass]
#[derive(Default, Debug)]
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
    /// Previous evaluation time
    prev_t: f64,
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

    pub fn update(&mut self, input: f64, t: f64) -> f64 {
        if (t - self.prev_t) >= self.T {
            let output = input * self.a[0] + self.inputs[0] * self.a[1] + self.inputs[1] * self.a[2]
                - self.outputs[0] * self.b[0]
                - self.outputs[1] * self.b[1];
            self.inputs[1] = self.inputs[0];
            self.inputs[0] = input;
            self.outputs[1] = self.outputs[0];
            self.outputs[0] = output;
            self.prev_t = t;
        }
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

#[derive(Default)]
pub struct Drone {
    config: Config,
    pid_velocity: RefCell<Pid>,
    pid_position: Option<RefCell<Pid>>,
    set_point: f64,
}

impl Environment for Drone {}

#[pyclass]
struct Model {
    config: Config,
}

#[pymethods]
impl Model {
    #[new]
    fn new(path: String) -> PyResult<Self> {
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
            w: config["hover"]["w"].as_float().ok_or(PyKeyError::new_err("hober/w"))?,
        };
        Ok(Self {
            config,
        })
    }

    #[pyo3(signature = (pid, save=false))]
    fn __call__(&self, pid: PyReadonlyArray1<f64>, save: bool) -> f64 {
        let kp = *pid.get(0).unwrap_or(&0.0);
        let ti = *pid.get(1).unwrap_or(&0.0);
        let td = *pid.get(2).unwrap_or(&0.0);
        let set_point = std::f64::consts::PI / 10.0;

        let drone = Drone {
            config: self.config,
            pid_velocity: RefCell::new(Pid::new(kp, ti, td, 5, 0.01)),
            pid_position: None,
            set_point,
        };

        let w = self.config.w;
        let state = State::<f64>::new(0f64, c!(w, w, w, w, 0, 0, 0, 0, 0, 0), c!(0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

        let mut ode_solver = ExplicitODE::new(compute_accel);

        ode_solver
            .set_method(ExMethod::RK4)
            .set_initial_condition(state)
            .set_env(drone)
            .set_stop_condition(|ode| {
                ode.get_state().value[0] > 800.0
                    || ode.get_state().value[0] < 0.0
                    || ode.get_state().value[1] > 800.0
                    || ode.get_state().value[1] < 0.0
            })
            .set_step_size(0.001)
            .set_times(1000);
        let result = ode_solver.integrate();
        if save {
            result.write("result.csv").expect("Could not open result.csv");
        }

        let err: f64 = result
            .col(5)
            .into_iter()
            .map(|y| set_point - y)
            .zip(result.col(0))
            .map(|(e, t)| e.abs() * t)
            .sum::<f64>()
            // Part of the itae missing due to early exit
            + (result.row..1001)
                .map(|x| set_point * f64::from(x as i16) * 0.01)
                .sum::<f64>();
        err
    }
}
/**
 * 0 Motor 0 | 4 Wx | 7 Px
 * 1 Motor 1 | 5 Wy | 8 Py
 * 2 Motor 2 | 6 Wz | 9 Pz
 * 3 Motor 3 |
 */
pub fn compute_accel(state: &mut State<f64>, env: &Drone) {
    // PID wx

    let cmd_px = env
        .pid_position
        .as_ref()
        .map(|pid| pid.borrow_mut().update(env.set_point - state.value[7], state.param))
        .unwrap_or(env.set_point);
    let cmd_wx = env.pid_velocity.borrow_mut().update(cmd_px - state.value[4], state.param);

    // The output of the PID should be between 0 and 200_000 to control the pwm
    // The throttle is between 0 and 1 so dividing by 200_000 does the trick
    let throttles = [
        env.config.throttle + (cmd_wx + 0.0 + 0.0) / 200_000.0,
        env.config.throttle + (-cmd_wx + 0.0 - 0.0) / 200_000.0,
        env.config.throttle + (-cmd_wx - 0.0 + 0.0) / 200_000.0,
        env.config.throttle + (cmd_wx - 0.0 - 0.0) / 200_000.0,
    ];

    for i in 0..4 {
        state.deriv[i] = (env.config.cr * throttles[i] + env.config.wb - state.value[i]) / env.config.tm;
    }

    let w0 = state.value[0].powi(2);
    let w1 = state.value[1].powi(2);
    let w2 = state.value[2].powi(2);
    let w3 = state.value[3].powi(2);
    let d = (2.0.sqrt() / 2.0) * env.config.size;
    // Wx
    state.deriv[4] = d * env.config.ct * (w0.pow(2.0) - w1.pow(2.0) - w2.pow(2.0) + w3.pow(2.0)) / env.config.jx;
    // Wy
    state.deriv[5] = d * env.config.ct * (w0.pow(2.0) + w1.pow(2.0) - w2.pow(2.0) - w3.pow(2.0)) / env.config.jy;
    // Wz
    state.deriv[6] = env.config.cm * (w0.pow(2.0) - w1.pow(2.0) + w2.pow(2.0) - w3.pow(2.0)) / env.config.jz;

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
