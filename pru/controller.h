#ifndef __CONTROLER_H__
#define __CONTROLER_H__
#pragma RESET_MISRA("all")

/* PRU CONSTANT */
#define PRU_INT_ECAP    15U
#define PRU_INT_HOST_3  16U
#define PRU_INT_HOST_1  21U  

#define ECAP_APWM_MODE  ((uint16_t)0x200U)
#define ECAP_TSCTRSTOP  ((uint16_t)0x10U)

#define ECAP_INT_CMPEQ  ((uint16_t)0x80U)

#endif
