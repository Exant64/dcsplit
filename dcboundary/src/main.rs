use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    // get args
    let args = env::args();
    let arg_count = args.len();
    if arg_count != 4 {
        panic!("USAGE: dcboundary target_bin key code_seg_end (ex. dcboundary 1ST_READ.bin 8C010000 5000)");
    }

    // parsing args
    let arg_strings : Vec<String> = args.collect();
        
    let key: usize = usize::from_str_radix(&arg_strings[2], 16)
        .expect("Invalid formatting for key argument!");

    let code_seg_end = u64::from_str_radix(&arg_strings[3], 16)
        .expect("Invalid formatting for code segment argument!");

    // file boundaries are 32 byte alignments
    let chunk_size = 32;    
    let mut last_start = 0;
    let mut code_seg_data = Vec::with_capacity(code_seg_end.try_into().unwrap());

    File::open(&arg_strings[1])
        .expect("Failed to open input file!")
        .take(code_seg_end)
        .read_to_end(&mut code_seg_data)
        .expect("Failed to read input file!");
    
    // we find mov.l instructions here to get the labels they refer to, to filter out false positive boundaries
    let instruction_size = 2;
    let data_l_iter = code_seg_data.chunks(instruction_size)
        .map(|arr| u16::from_le_bytes([arr[0], arr[1]]))
        .filter(|x: &u16| (*x & 0xF000) == 0xD000)
        .enumerate()
        .map(|(index, instr)| {
            let disp: usize = (instr & 0x00FF) as usize;
            let pc: usize = index * 2 + key;

            // the label's address
            (pc & 0xFFFFFFFC) + 4 + (disp << 2)
        });

    let data_l_labels: HashSet<usize> = HashSet::from_iter(data_l_iter);

    for (i, chunk) in code_seg_data.chunks(chunk_size).enumerate() {
        let end_of_chunk = (i+1) * chunk_size;

        // if the 32 byte chunk ends with an int data label, it's definitely not padding
        if data_l_labels.contains(&(end_of_chunk - 4 + key)) {
            continue;
        }

        let count = chunk.into_iter()
                                .rev()
                                .take_while(|x| **x == 0)
                                .count();
                                          
        // if it's a valid boundary
        if count > 1 && count < chunk_size {
            println!("- [{:#x}, asm]", last_start);
            last_start = end_of_chunk;
        }
    }
}
