use primitive_types::U256;

pub fn is_valid_jump_dest(code: &[u8], jump_dest: usize) -> bool {
    // If jump destination is out of bounds, return false
    if jump_dest >= code.len() {
        return false;
    }

    let mut pc = 0;
    while pc < code.len() {
        let opcode = code[pc];

        // If opcode is JUMPDEST, check if pc is the jump destination
        if opcode == 0x5b && pc == jump_dest {
            return true;
        }

        // If opcode is PUSH1-PUSH32, skip the next N bytes
        if opcode >= 0x60 && opcode <= 0x7f {
            let data_size = (opcode - 0x60) + 1;
            pc += data_size as usize;
        }

        pc += 1;
    }

    false
}

pub fn sdiv(mut a: U256, mut b: U256) -> U256 {
    let mask = U256::one() << 255;
    let a_flag = a & mask;
    let b_flag = b & mask;

    if a_flag & b_flag == mask {
        let a = a.overflowing_neg().0;
        let b = b.overflowing_neg().0;
        a.checked_div(b).or_else(|| Some(U256::zero())).unwrap()
    } else if a_flag == U256::zero() && b_flag == U256::zero() {
        a.checked_div(b).or_else(|| Some(U256::zero())).unwrap()
    } else {
        if a_flag == U256::zero() {
            b = b.overflowing_neg().0;
        } else {
            a = a.overflowing_neg().0;
        }
        let val = a.checked_div(b).or_else(|| Some(U256::zero())).unwrap();
        val.overflowing_neg().0
    }
}

pub fn smod(mut a: U256, mut b: U256) -> U256 {
    let mask = U256::one() << 255;
    let a_flag = a & mask;
    let b_flag = b & mask;

    if a_flag == U256::zero() && b_flag == U256::zero() {
        a.checked_rem(b).or_else(|| Some(U256::zero())).unwrap()
    } else {
        if a_flag == U256::zero() {
            b = b.overflowing_neg().0;
        } else if b_flag == U256::zero() {
            a = a.overflowing_neg().0;
        } else {
            a = a.overflowing_neg().0;
            b = b.overflowing_neg().0;
        }
        let val = a.checked_rem(b).or_else(|| Some(U256::zero())).unwrap();
        val.overflowing_neg().0
    }
}

pub fn slt(mut a: U256, mut b: U256) -> U256 {
    let mask = U256::one() << 255;
    let a_flag = a & mask;
    let b_flag = b & mask;

    if a_flag == U256::zero() && b_flag == U256::zero() {
        if a < b {
            U256::one()
        } else {
            U256::zero()
        }
    } else if a_flag == U256::zero() {
        U256::zero()
    } else if b_flag == U256::zero() {
        U256::one()
    } else {
        a = a.overflowing_neg().0;
        b = b.overflowing_neg().0;
        if a < b {
            U256::one()
        } else {
            U256::zero()
        }
    }
}

pub fn sgt(mut a: U256, mut b: U256) -> U256 {
    let mask = U256::one() << 255;
    let a_flag = a & mask;
    let b_flag = b & mask;

    if a_flag == U256::zero() && b_flag == U256::zero() {
        if a > b {
            U256::one()
        } else {
            U256::zero()
        }
    } else if a_flag == U256::zero() {
        U256::zero()
    } else if b_flag == U256::zero() {
        U256::one()
    } else {
        a = a.overflowing_neg().0;
        b = b.overflowing_neg().0;
        if a < b {
            U256::one()
        } else {
            U256::zero()
        }
    }
}

pub fn signextend(k: U256, v: U256) -> U256 {
    if k < U256::from(32) {
        let bit_position = k.as_usize() * 8 + 7;
        let mask = (U256::one() << bit_position) - U256::one();

        if v.bit(bit_position) {
            v | !mask
        } else {
            v & mask
        }
    } else {
        v
    }
}

pub fn sar(a: U256, b: U256) -> U256 {
    let mask = U256::one() << 255;
    let msb_is_zero = b & mask == U256::zero();
    if a >= U256::from(256) {
        if msb_is_zero {
            U256::zero()
        } else {
            U256::MAX
        }
    } else {
        if msb_is_zero {
            b >> a
        } else {
            let v_zero = b >> a;
            let mask_one = U256::MAX << (U256::from(256) - a);
            mask_one | v_zero
        }
    }
}
