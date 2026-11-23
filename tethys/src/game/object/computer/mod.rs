/*
    pub fn tick(&mut self, block: &mut CommandBlockInfo) -> Result<(), MachineError> {
        let mut message_index = 0;
        while let Some(function) = self.run_to_call()? {
            match function {
                0 => {// Print
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    println!("Breakpoint {}", arg);
                },
                1 => {// Add force
                    let mut force = Vector3::new(
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?
                    );
                    force = block.quat.rotate_vector(force);
                    let torque = block.pos.cross(force);
                    let force = block.body.ori.rotate_vector(force);
                    block.body.add_force(force);
                    block.body.add_torque(torque);
                },
                2 => {// Add torque
                    let torque = Vector3::new(self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?);
                    block.body.add_torque(torque);
                },
                3 => {// Emit signal
                    let recv_block = self.stack.pop().ok_or(MachineError::Stack)?.round() as u8;
                    let n_send = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize;
                    
                    let mut data = Vec::with_capacity(n_send);
                    for _ in 0..n_send {
                        data.push(self.stack.pop().ok_or(MachineError::Stack)?);
                    }
                    
                    match &mut block.circuit {
                        Some(c) => {
                            c.send(recv_block, data)
                        },
                        None => (),
                    }
                },
                4 => {// Receive signal
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    let jtrue = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    let jfalse = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    match &block.circuit {
                        Some(c) => {
                            let (data_option, new_message_index) = c.recv(block.id, message_index);
                            match data_option {
                                Some(data) => {
                                    for d in data {
                                        self.stack.push(*d);
                                    }
                                    self.stack.push(data.len() as f64);
                                    self.ip = jtrue;
                                },
                                None => {
                                    self.ip = jfalse;
                                }
                            }
                            message_index = new_message_index;
                        },
                        None => self.ip = jfalse,
                    }
                },
                _ => return Err(MachineError::Func)
            }
        }
        Ok(())
    } */