use hil::{GPIOPin};

pub enum LEDStatus {
    Off, On
}

pub struct LEDParams {
    pub start_status: LEDStatus,
}

pub struct LED<P: GPIOPin> {
    pin: P,
    status: LEDStatus
}

impl<P: GPIOPin> LED<P> {
    pub fn new(mut pin: P, params: LEDParams) -> LED<P> {
        pin.enable_output();
        if let LEDStatus::On = params.start_status {
            pin.toggle();
        }

        LED {
            pin: pin,
            status: params.start_status
        }
    }

    pub fn toggle(&mut self) {
        self.status = match self.status {
            LEDStatus::On => LEDStatus::Off,
            LEDStatus::Off => LEDStatus::On
        };

        self.pin.toggle();
    }

    pub fn on(&mut self) {
        if let LEDStatus::Off = self.status {
            self.toggle();
        }
    }

    pub fn off(&mut self) {
        if let LEDStatus::On = self.status {
            self.toggle();
        }
    }
}
