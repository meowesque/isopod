use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("new-isopod-lib.iso")?;
    
    // Read first sector (2048 bytes)
    let mut first_sector = [0u8; 2048];
    file.read_exact(&mut first_sector)?;
    
    println!("Volume Descriptor Analysis:");
    
    // Check volume descriptor type
    println!("Volume Descriptor Type: {}", first_sector[0]);
    
    // Check standard identifier
    let standard_id = &first_sector[1..6];
    println!("Standard ID: {}", 
        String::from_utf8_lossy(standard_id)
    );
    
    // Extract volume identifier
    let volume_id = String::from_utf8_lossy(&first_sector[40..72]).trim().to_string();
    println!("Volume ID: '{}'", volume_id);
    
    // Seek to primary volume descriptor (16th sector)
    file.seek(SeekFrom::Start(16 * 2048))?;
    file.read_exact(&mut first_sector)?;
    
    println!("\nPrimary Volume Descriptor Check:");
    println!("Volume Descriptor Type: {}", first_sector[0]);
    println!("Standard ID: {}", 
        String::from_utf8_lossy(&first_sector[1..6])
    );
    
    // Read path table location and size
    let path_table_size = u32::from_le_bytes([
        first_sector[132], 
        first_sector[133], 
        first_sector[134], 
        first_sector[135]
    ]);
    
    let path_table_location = u32::from_le_bytes([
        first_sector[140], 
        first_sector[141], 
        first_sector[142], 
        first_sector[143]
    ]);
    
    println!("Path Table Size: {} bytes", path_table_size);
    println!("Path Table Location: sector {}", path_table_location);
    
    Ok(())
}