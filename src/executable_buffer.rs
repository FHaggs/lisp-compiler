use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use std::mem;

pub struct ExecBuffer {
    memory: *mut libc::c_void,
    size: usize,
}
impl ExecBuffer {
    pub fn new(code: &[u8]) -> Result<Self, &'static str> {
        let size = code.len();
        unsafe {
            let memory = libc::mmap(
                std::ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE, // Start as writable
                MAP_ANONYMOUS | MAP_PRIVATE,
                -1,
                0,
            );

            if memory == libc::MAP_FAILED {
                return Err("mmap failed");
            }

            // Copy the machine code into the new memory.
            libc::memcpy(memory, code.as_ptr() as *const libc::c_void, size);

            // Mark the memory as executable.
            if libc::mprotect(memory, size, PROT_READ | PROT_EXEC) != 0 {
                // Clean up on failure
                libc::munmap(memory, size);
                return Err("mprotect failed");
            }

            Ok(ExecBuffer { memory, size })
        }
    }
    pub unsafe fn as_function<F: Copy>(&self) -> F {
        unsafe { mem::transmute_copy(&self.memory) }
    }
}

impl Drop for ExecBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.memory, self.size);
        }
    }
}

mod tests {
    use super::*;

    type TestJitFunction = unsafe extern "C" fn() -> i32;
    #[test]
    fn test_exec_buffer() {
        // Program that returns 42
        let my_program: Vec<u8> = vec![0xb8, 0x2a, 0x00, 0x00, 0x00, 0xc3];
        // translate_to_executable_buffer(my_program);
        if let Ok(exec_buff) = ExecBuffer::new(&my_program) {
            unsafe {
                let func = exec_buff.as_function::<TestJitFunction>();
                let res = func();
                println!("Result: {}", res);
                assert!(res == 42);
            }
        }
    }
}
