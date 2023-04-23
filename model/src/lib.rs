use numpy::PyReadonlyArray1;
use peroxide::c;
use peroxide::fuga::*;
use pyo3::prelude::*;
use std::cell::RefCell;

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
    }

    pub fn __str__(&self) -> String {
        format!("PID: a = {:?}, b = {:?}", self.a, self.b).to_string()
    }
}

impl Pid {
    pub fn update(&mut self, input: f64, t: f64) -> f64 {
        if (t - self.prev_t) >= self.T {
            let output = input * self.a[0]
                + self.inputs[0] * self.a[1]
                + self.inputs[1] * self.a[2]
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

#[derive(Default)]
pub struct Drone {
    pub tm: f64,
    pub cr: f64,
    pub wb: f64,
    pub d: f64,
    pub ct: f64,
    pub cm: f64,
    pub jx: f64,
    pub jy: f64,
    pub jz: f64,
    pub hover_throttle: f64,
    pub pid_velocity: RefCell<Pid>,
    pub set_point: f64,
}

impl Environment for Drone {}

/**
 * 0 Motor 0 | 4 Wx | 7 Px
 * 1 Motor 1 | 5 Wy | 8 Py
 * 2 Motor 2 | 6 Wz | 9 Pz
 * 3 Motor 3 |
 */
pub fn compute_accel(state: &mut State<f64>, env: &Drone) {
    // PID wx

    let cmd_wx = env
        .pid_velocity
        .borrow_mut()
        .update(env.set_point - state.value[4], state.param);

    // Motor 0
    state.deriv[0] = (env.cr * (env.hover_throttle + cmd_wx + 0.0 + 0.0)
        + env.wb
        - state.value[0])
        / env.tm;
    // Motor 1
    state.deriv[1] = (env.cr * (env.hover_throttle - cmd_wx + 0.0 - 0.0)
        + env.wb
        - state.value[1])
        / env.tm;
    // Motor 2
    state.deriv[2] = (env.cr * (env.hover_throttle - cmd_wx - 0.0 + 0.0)
        + env.wb
        - state.value[2])
        / env.tm;
    // Motor 3
    state.deriv[3] = (env.cr * (env.hover_throttle + cmd_wx - 0.0 - 0.0)
        + env.wb
        - state.value[3])
        / env.tm;

    let w0 = state.value[0].powi(2);
    let w1 = state.value[1].powi(2);
    let w2 = state.value[2].powi(2);
    let w3 = state.value[3].powi(2);
    // Wx
    state.deriv[4] = env.d * env.ct * (w0 - w1 - w2 + w3) / env.jx;
    // Wy
    state.deriv[5] = env.d * env.ct * (w0 + w1 - w2 - w3) / env.jy;
    // Wz
    state.deriv[6] = env.cm * (w0 - w1 + w2 - w3) / env.jz;

    // Px
    state.deriv[7] = state.value[4];
    // Py
    state.deriv[8] = state.value[5];
    // Pz
    state.deriv[9] = state.value[6];
}

#[pyfunction(save = "false")]
fn pid_velocity_x(pid: PyReadonlyArray1<f64>, save: bool) -> f64 {
    let kp = *pid.get(0).unwrap_or(&0.0);
    let ti = *pid.get(1).unwrap_or(&0.0);
    let td = *pid.get(2).unwrap_or(&0.0);
    let set_point = std::f64::consts::PI / 10.0;
    let drone = Drone {
        tm: 0.0164,
        cr: 751.64,
        wb: 63.61,
        d: 0.45 * 2.0.sqrt() / 2.0,
        ct: 1.2015e-5,
        cm: 2.1057e-7,
        jx: 0.01334,
        jy: 0.01334,
        jz: 0.02557,
        hover_throttle: 0.48,
        pid_velocity: RefCell::new(Pid::new(kp, ti, td, 5, 0.01)),
        set_point: 1.6,
    };
    if save {
        println!("{:#?}", drone.pid_velocity);
    }

    let state = State::<f64>::new(
        0f64,
        c!(428.39, 428.39, 428.39, 428.39, 0, 0, 0, 0, 0, 0),
        c!(0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
    );

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
        .map(|y| 1.6 - y)
        .zip(result.col(0).into_iter())
        .map(|(e, t)| e.abs() * t)
        .sum::<f64>()
        // Part of the itae missing due to early exit
        + (result.row..1001)
            .map(|x| 1.6 * f64::from(x as i16) * 0.01)
            .sum::<f64>();
    err
}

#[pymodule]
fn model(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pid_velocity_x, m)?)?;
    m.add_function(wrap_pyfunction!(pid_position_x, m)?)?;
    m.add_class::<Pid>()?;
    Ok(())
}
