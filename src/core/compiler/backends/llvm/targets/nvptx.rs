use crate::core::console::logging;

#[inline]
pub fn print_all_supported_cpus() {
    logging::write(
        logging::OutputIn::Stdout,
        "sm_100
sm_100a
sm_101
sm_101a
sm_120
sm_120a
sm_20
sm_21
sm_30
sm_32
sm_35
sm_37
sm_50
sm_52
sm_53
sm_60
sm_61
sm_62
sm_70
sm_72
sm_75
sm_80
sm_86
sm_87
sm_89
sm_90",
    );
}
