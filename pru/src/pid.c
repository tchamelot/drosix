#include <string.h>
#include <stdint.h>
#include "pid.h"

void pid_init(struct pid_controller_t* pid, const volatile pid_config_t* config, const float sampling) {
    float kp, ti, td, n, kaw, t;
    float  div;


    kp = config->kpr;
    ti = config->ti;
    td = config->td;
    n = config->filter;
    kaw = config->kaw;
    t = sampling;

    // TODO check t != 0 

    // Proportional Derivative
    if(td != 0.0) {
        div = n*t + 2*td;
        pid->kd[0] = kp * (n*t + 2*n*td + 2*td) / div;
        pid->kd[1] = kp * (n*t - 2*n*td - 2*td) / div;
        pid->kd[2] = (n*t - 2*td) / div;
    }
    else {
        pid->kd[0] = kp;
        pid->kd[1] = 0.0;
        pid->kd[2] = 0.0;
    }

    // Intergative anti-windup
    if(ti != 0.0) {
        pid->ki = (kp*t) / (2*ti);
        pid->kaw = kaw;
    }
    else {
        pid->ki = 0.0;
        pid->kaw = 0.0;
    }

    pid->max = config->max;
    pid->min = config->min;

    pid_reset(pid);
}

void pid_reset(struct pid_controller_t* pid) {
    pid->d_out_prev = 0.0;
    pid->i_out_prev = 0.0;
    pid->input_prev = 0.0;
    pid->sat_err_prev[0] = 0.0;
    pid->sat_err_prev[1] = 0.0;
}

float pid_run(struct pid_controller_t* pid, float input) {
    float out_pd, out_i, out_pid, output;
    float i_in;

    out_pd = input * pid->kd[0] + pid->input_prev * pid->kd[1] - pid->d_out_prev * pid->kd[2];
    i_in = input + pid->sat_err_prev[0] * pid->kaw;
    out_i = pid->ki * (i_in + pid->sat_err_prev[1]) + pid->i_out_prev;

    out_pid = out_pd + out_i;

    if(out_pid > pid->max) {
        output = pid->max;
    } else if(out_pid < pid->min) {
        output = pid->min;
    } else {
        output = out_pid;
    }

    pid->sat_err_prev[0] = output - out_pid;
    pid->sat_err_prev[1] = i_in;
    pid->d_out_prev = out_pd;
    pid->i_out_prev = out_i;
    pid->input_prev = input;

    return output;
}
