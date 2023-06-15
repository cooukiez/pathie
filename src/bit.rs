#[macro_export]
macro_rules! bitset {
    ($num : expr, $bit : expr) => {
        $num | 1 << $bit
    };
}

#[macro_export]
macro_rules! bitclear {
    ($num : expr, $bit : expr) => {
        $num & !(1 << $bit)
    };
}

#[macro_export]
macro_rules! bitflip {
    ($num : expr, $bit : expr) => {
        $num ^ 1 << $bit
    };
}

#[macro_export]
macro_rules! bitcheck {
    ($num : expr, $bit : expr) => {
        ($num >> $bit) & 1
    };
}

#[macro_export]
macro_rules! set_bit {
    ($value : expr, $bit : expr, $bit_value : expr) => {{
        if $bit_value == true {
            $value | (1 << $bit)
        } else {
            $value & !(1 << $bit)
        }
    }};
}

#[macro_export]
macro_rules! read_bitrange {
    ($num : expr, $s : expr, $e : expr) => {{
        let mask = (!0u32 >> $e) << ($s - 1);
        ($num & mask) >> ($s - 1)
    }};
}

#[macro_export]
macro_rules! write_bitrange {
    ($num : expr, $rep : expr, $s : expr, $e : expr) => {{
        let mask = (!0u32 >> $e) << ($s - 1);
        ($num & !(mask)) | ($rep & mask)
    }};
}