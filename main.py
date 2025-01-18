import mido

def list_and_choose_midi_ports():
    """List all available MIDI ports and let user choose."""
    input_ports = mido.get_input_names()
    output_ports = mido.get_output_names()
    
    print("\nAvailable MIDI Input Ports:")
    for i, port in enumerate(input_ports):
        print(f"{i}: {port}")
    
    print("\nAvailable MIDI Output Ports:")
    for i, port in enumerate(output_ports):
        print(f"{i}: {port}")
    
    try:
        input_choice = int(input("\nChoose input port number: "))
        output1_choice = int(input("Choose first output port number: "))
        output2_choice = int(input("Choose second output port number: "))
        
        return (input_ports[input_choice], 
                output_ports[output1_choice],
                output_ports[output2_choice])
    except (ValueError, IndexError) as e:
        print(f"Error choosing ports: {e}")
        return None, None, None

def handle_message(msg, outport1, outport2):
    # Print raw MIDI data for debugging
    print(f"Received MIDI message: type={msg.type}, ", end='')
    if hasattr(msg, 'control'):
        print(f"control={msg.control}, value={msg.value}")
    else:
        print(f"data={msg}")
    
    # Forward messages immediately with no delay
    outport1.send(msg.copy(time=0))
    outport2.send(msg.copy(time=0))

def get_scale_notes(key):
    """Returns the notes in the major scale for the given key"""
    major_scale_steps = [0, 2, 4, 5, 7, 9, 11]
    scale_notes = [(key + step) % 12 for step in major_scale_steps]
    return scale_notes

def find_nearest_diatonic_third(note, key):
    """Find the nearest diatonic third above the note in the given key"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Find the note two scale degrees above
    base_index = scale_notes.index(base_note)
    third_index = (base_index + 2) % 7
    third_note = scale_notes[third_index]
    
    # Adjust octave
    while third_note <= base_note:
        third_note += 12
    
    return int(third_note + (note // 12) * 12)

def get_counterpoint_motion(prev_base, prev_counter, new_base, scale_notes):
    """Determine best counterpoint motion following basic rules:
    - Prefer contrary motion
    - Avoid parallel fifths and octaves
    - Stay within scale
    - Prefer consonant intervals (thirds, sixths, perfect fifths/octaves)
    - Avoid unison except at start/end
    """
    consonant_intervals = [3, 4, 7, 8, 9]  # Minor third to sixth, perfect fifth
    base_direction = 1 if new_base > prev_base else -1
    
    # Get all possible scale notes within reasonable range
    possible_notes = []
    for octave in [-1, 0, 1]:
        for note in scale_notes:
            possible_notes.append(note + (12 * octave))
    
    # Filter to notes within reasonable range of new base note
    possible_notes = [n for n in possible_notes if -12 <= (n - new_base) <= 12]
    
    # Remove the input note from possibilities (avoid unison)
    possible_notes = [n for n in possible_notes if (n % 12) != (new_base % 12)]
    
    # Score each possible note
    best_score = -float('inf')
    best_note = None
    
    for note in possible_notes:
        score = 0
        interval = abs(note - new_base) % 12
        
        # Prefer contrary motion
        if (note - prev_counter) * base_direction < 0:
            score += 3
            
        # Avoid parallel fifths/octaves
        prev_interval = abs(prev_counter - prev_base) % 12
        if interval in [7, 0] and interval == prev_interval:
            score -= 10
            
        # Prefer consonant intervals
        if interval in consonant_intervals:
            score += 2
            
        # Prefer thirds and sixths over fifths and octaves
        if interval in [3, 4, 8, 9]:  # thirds and sixths
            score += 1
            
        # Prefer smaller leaps
        leap_size = abs(note - prev_counter)
        if leap_size <= 2:
            score += 2
        elif leap_size <= 4:
            score += 1
        elif leap_size >= 8:
            score -= 2
            
        if score > best_score:
            best_score = score
            best_note = note
            
    # If no good options found, default to a third above
    if best_note is None:
        base_scale_pos = scale_notes.index(new_base % 12)
        third_scale_pos = (base_scale_pos + 2) % len(scale_notes)
        best_note = scale_notes[third_scale_pos]
        while best_note <= new_base:
            best_note += 12
            
    return int(best_note)

def main():
    # Get available ports
    available_ports = mido.get_input_names()
    output_ports = mido.get_output_names()
    
    print("\nAvailable MIDI input ports:")
    for i, port in enumerate(available_ports):
        print(f"{i}: {port}")

    # Select input port
    port_num = int(input("\nSelect input port number: "))
    input_port = mido.open_input(available_ports[port_num])

    # Select output ports
    print("\nAvailable MIDI output ports:")
    for i, port in enumerate(output_ports):
        print(f"{i}: {port}")
    
    output1_num = int(input("\nSelect first output port number: "))
    output2_num = int(input("Select second output port number: "))

    # Select key
    key_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    print("\nSelect key:")
    for i, key in enumerate(key_names):
        print(f"{i}: {key}")
    key = int(input("Enter key number: "))

    # Select mode
    print("\nSelect mode:")
    print("1: Forward MIDI as-is")
    print("2: Add diatonic thirds")
    print("3: Generate counterpoint")
    mode = int(input("Enter mode number: "))

    # Open output ports
    output_port1 = mido.open_output(output_ports[output1_num])
    output_port2 = mido.open_output(output_ports[output2_num])

    print(f"\nInput: {available_ports[port_num]}")
    print(f"Output 1: {output_ports[output1_num]}")
    print(f"Output 2: {output_ports[output2_num]}")
    print("\nListening for MIDI messages... Press Ctrl+C to exit.")

    prev_base_note = None
    prev_counter_note = None
    scale_notes = get_scale_notes(key) if mode in [2, 3] else None

    try:
        for msg in input_port:
            if msg.type == 'note_on' or msg.type == 'note_off':
                # Always send original note to first output
                output_port1.send(msg)

                if mode == 1:
                    # Forward as-is
                    output_port2.send(msg)
                elif mode == 2:
                    # Add diatonic third
                    third_note = find_nearest_diatonic_third(msg.note, key)
                    third_msg = msg.copy(note=int(third_note))
                    output_port2.send(third_msg)
                else:  # Counterpoint mode
                    if prev_base_note is None:
                        # For first note, use a third or fifth above
                        counter_note = find_nearest_diatonic_third(msg.note, key)
                    else:
                        counter_note = get_counterpoint_motion(
                            prev_base_note, 
                            prev_counter_note, 
                            msg.note, 
                            scale_notes
                        )
                    
                    counter_msg = msg.copy(note=int(counter_note))
                    output_port2.send(counter_msg)
                    
                    if msg.type == 'note_on':
                        prev_base_note = msg.note
                        prev_counter_note = int(counter_note)
            else:
                # Forward all other MIDI messages to both outputs
                output_port1.send(msg)
                output_port2.send(msg)

    except KeyboardInterrupt:
        print("\nExiting...")
        input_port.close()
        output_port1.close()
        output_port2.close()

if __name__ == '__main__':
    main()
