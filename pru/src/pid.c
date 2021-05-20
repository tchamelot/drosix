#include "pid.h"

float run_pid(struct pid_t* pid, float error) {
    float p, i_min, i_max;

    // Proportional
    p = error * pid->kp;

    // Integral
    pid->i_prev = pid->ki * (error + pid->error_prev) + pid->i_prev; 
    // Integral clamping
    if(p < 1) {
        i_max = 1.f - p;
    }
    else {
        i_max = 0.f;
    }
    if(p > 0) {
        i_min = 0.f - p;
    }
    else {
        i_min = 0.f;
    }
    if(pid->i_prev > i_max) {
        pid->i_prev = i_max;
    }
    if(pid->i_prev < i_min) {
        pid->i_prev = i_min;
    }

    // Derivate
    pid->d_prev = pid->kd1 * (pid->error_prev - error) + pid->kd2 * pid->d_prev;

    // Output
    float res = p + pid->i_prev + pid->d_prev;

    // Update error
    pid->error_prev = error;

    // Output clamping
    if(res < 0) {
        res = 0;
    }
    if(res > 1) {
        res = 1;
    }
    return res;
}
