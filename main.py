import mido
import random
import sys
import termios
import tty
import threading
import queue

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

def get_scale_notes(key):
    """Returns the notes in the major scale for the given key"""
    major_scale_steps = [0, 2, 4, 5, 7, 9, 11]
    scale_notes = [(key + step) % 12 for step in major_scale_steps]
    return scale_notes

def find_nearest_diatonic_third(note, key):
    """Find the nearest diatonic third below the note in the given key"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Find the note two scale degrees below
    base_index = scale_notes.index(base_note)
    third_index = (base_index - 2) % 7  # Changed from +2 to -2 to go down
    third_note = scale_notes[third_index]
    
    # Adjust octave to be below the input note
    current_octave = note // 12
    third_note = third_note + (current_octave * 12)
    
    # If the third is still above or equal to the input note, move it down an octave
    if third_note >= note:
        third_note -= 12
    
    return int(third_note)

def find_random_diatonic_below(note, key):
    """Find a random diatonic note below the input note, within an octave range"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Get all possible scale notes within one octave below
    possible_notes = []
    current_octave = note // 12
    
    # Add notes from current octave and one octave below
    for octave in [current_octave - 1, current_octave]:
        for scale_note in scale_notes:
            note_value = scale_note + (12 * octave)
            if note_value < note:  # Only include notes below input
                possible_notes.append(note_value)
    
    if not possible_notes:  # If no notes found, take one octave below input
        return note - 12
        
    return random.choice(possible_notes)

def find_random_diatonic_below_no_seconds(note, key):
    """Find a random diatonic note below the input note, excluding seconds"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Get all possible scale notes within one octave below
    possible_notes = []
    current_octave = note // 12
    
    # Add notes from current octave and one octave below
    for octave in [current_octave - 1, current_octave]:
        for scale_note in scale_notes:
            note_value = scale_note + (12 * octave)
            # Only include notes that are:
            # 1. Below the input note
            # 2. Not a minor second (1 semitone) or major second (2 semitones) away
            interval = abs(note - note_value) % 12
            if note_value < note and interval not in [1, 2]:
                possible_notes.append(note_value)
    
    if not possible_notes:  # If no notes found, take one octave below input
        return note - 12
        
    return random.choice(possible_notes)

def find_contrary_diatonic_below_no_seconds(note, key, prev_input=None, prev_output=None):
    """Find a random diatonic note below the input note that moves in contrary motion"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # If no previous notes, just use regular random function
    if prev_input is None or prev_output is None:
        return find_random_diatonic_below_no_seconds(note, key)
    
    # Determine input direction
    input_direction = 1 if note > prev_input else (-1 if note < prev_input else 0)
    
    # If input note didn't change, use regular random function
    if input_direction == 0:
        return find_random_diatonic_below_no_seconds(note, key)
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Get all possible scale notes within one octave below
    possible_notes = []
    current_octave = note // 12
    
    # Add notes from current octave and one octave below
    for octave in [current_octave - 1, current_octave]:
        for scale_note in scale_notes:
            note_value = scale_note + (12 * octave)
            # Only include notes that are:
            # 1. Below the input note
            # 2. Not a minor second (1 semitone) or major second (2 semitones) away
            # 3. Moving in contrary motion to the input
            interval = abs(note - note_value) % 12
            note_direction = 1 if note_value > prev_output else (-1 if note_value < prev_output else 0)
            if (note_value < note and 
                interval not in [1, 2] and 
                note_direction * input_direction < 0):  # Ensures contrary motion
                possible_notes.append(note_value)
    
    if not possible_notes:  # If no contrary motion notes found, fall back to regular function
        return find_random_diatonic_below_no_seconds(note, key)
        
    return random.choice(possible_notes)

def find_strict_counterpoint_below(note, key, prev_input=None, prev_output=None):
    """Generate counterpoint following standard rules:
    1. Prefer contrary motion
    2. Use consonant intervals (3rds, 6ths, 5ths, octaves)
    3. No parallel fifths or octaves
    4. No direct fifths or octaves
    5. Resolve leading tones properly
    6. Step motion after leaps
    7. Limited range between voices
    """
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # If no previous notes, start with a consonant interval
    if prev_input is None or prev_output is None:
        consonant_intervals = [3, 4, 7, 8, 9]  # thirds, fifth, sixth
        possible_notes = []
        current_octave = note // 12
        
        for octave in [current_octave - 1, current_octave]:
            for scale_note in scale_notes:
                note_value = scale_note + (12 * octave)
                interval = abs(note - note_value) % 12
                if note_value < note and interval in consonant_intervals:
                    possible_notes.append(note_value)
        
        return random.choice(possible_notes) if possible_notes else (note - 12)
    
    # Determine input direction
    input_direction = 1 if note > prev_input else (-1 if note < prev_input else 0)
    
    # Get all possible scale notes within range
    possible_notes = []
    current_octave = note // 12
    
    for octave in [current_octave - 1, current_octave]:
        for scale_note in scale_notes:
            note_value = scale_note + (12 * octave)
            
            # Skip if note is above input or too far below (more than 12th)
            if note_value >= note or (note - note_value) > 19:
                continue
                
            # Calculate intervals
            curr_interval = abs(note - note_value) % 12
            prev_interval = abs(prev_input - prev_output) % 12
            
            # Skip if creates parallel fifths or octaves
            if curr_interval in [7, 0] and curr_interval == prev_interval:
                continue
                
            # Calculate motion direction
            note_direction = 1 if note_value > prev_output else (-1 if note_value < prev_output else 0)
            
            # Score this note
            score = 0
            
            # Prefer contrary motion
            if note_direction * input_direction < 0:
                score += 5
                
            # Consonant intervals
            if curr_interval in [3, 4, 8, 9]:  # thirds and sixths
                score += 4
            elif curr_interval in [7, 0]:  # fifths and octaves
                score += 2
                
            # Proper resolution of leading tones
            prev_scale_pos = scale_notes.index(prev_output % 12)
            curr_scale_pos = scale_notes.index(note_value % 12)
            if prev_scale_pos == 6:  # If previous was leading tone
                if curr_scale_pos == 0:  # Resolves to tonic
                    score += 6
                    
            # Step motion after leaps
            prev_leap = abs(prev_output - prev_input)
            curr_step = abs(note_value - prev_output)
            if prev_leap > 4 and curr_step <= 2:  # Step after leap
                score += 4
                
            # Add note if it passes basic rules
            if score > 0:
                possible_notes.append((note_value, score))
    
    if not possible_notes:  # If no valid notes found, fall back to basic contrary motion
        return find_contrary_diatonic_below_no_seconds(note, key, prev_input, prev_output)
        
    # Choose note weighted by scores
    total_score = sum(score for _, score in possible_notes)
    choice = random.uniform(0, total_score)
    current = 0
    for note_value, score in possible_notes:
        current += score
        if current >= choice:
            return note_value
            
    # Fallback if something goes wrong
    return possible_notes[0][0]

def get_key_nonblocking():
    """Get a single keypress without blocking."""
    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)
    try:
        tty.setraw(sys.stdin.fileno())
        ch = sys.stdin.read(1) if sys.stdin.readable() else None
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
    return ch

def keyboard_input_thread(command_queue):
    """Thread function to handle keyboard input."""
    while True:
        key = get_key_nonblocking()
        if key:
            if key in ['1', '2', '3', '4', '5', '6']:  # Added '6' to valid mode keys
                command_queue.put(('change_mode', int(key)))
            elif key == 'q':
                command_queue.put(('quit', None))
                break

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

    # Select number of outputs
    print("\nHow many outputs do you want? (minimum 2)")
    num_outputs = max(2, int(input("Enter number of outputs: ")))

    # Select output ports
    print("\nAvailable MIDI output ports:")
    for i, port in enumerate(output_ports):
        print(f"{i}: {port}")
    
    output_nums = []
    output_ports_list = []
    
    print("\nSelect output port numbers:")
    for i in range(num_outputs):
        if i == 0:
            port_num = int(input(f"Select output port {i+1} (melody): "))
        else:
            port_num = int(input(f"Select output port {i+1} (harmony): "))
        output_nums.append(port_num)
        output_ports_list.append(mido.open_output(output_ports[port_num]))

    # Select key
    key_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    print("\nSelect key:")
    for i, key in enumerate(key_names):
        print(f"{i}: {key}")
    key = int(input("Enter key number: "))

    # Select initial mode
    print("\nSelect mode:")
    print("1: Forward MIDI as-is")
    print("2: Add diatonic thirds")
    print("3: Add random diatonic intervals")
    print("4: Add random diatonic intervals (no seconds)")
    print("5: Add contrary motion with random intervals (no seconds)")
    print("6: Add strict counterpoint rules")
    print("\nYou can change modes during runtime using number keys 1-6")
    print("Press 'q' to quit")
    mode = int(input("Enter mode number: "))

    print(f"\nInput: {available_ports[port_num]}")
    print("Outputs:")
    for i, port_num in enumerate(output_nums):
        print(f"Output {i+1}: {output_ports[port_num]}")
    print("\nListening for MIDI messages...")
    print("Current mode:", mode)
    print("Press 1-6 to change modes, 'q' to quit")

    scale_notes = get_scale_notes(key)
    active_notes = {}  # Maps input note to generated note
    prev_input_note = None  # Track previous input note for contrary motion
    prev_output_note = None  # Track previous output note for contrary motion
    
    # Set up command queue and keyboard input thread
    command_queue = queue.Queue()
    keyboard_thread = threading.Thread(target=keyboard_input_thread, args=(command_queue,))
    keyboard_thread.daemon = True
    keyboard_thread.start()

    try:
        while True:
            # Check for mode change commands
            try:
                command, value = command_queue.get_nowait()
                if command == 'change_mode':
                    mode = value
                    # Reset motion tracking when changing modes
                    prev_input_note = None
                    prev_output_note = None
                    print(f"\nChanged to mode {mode}")
                elif command == 'quit':
                    raise KeyboardInterrupt
            except queue.Empty:
                pass

            # Handle MIDI messages
            msg = input_port.poll()
            if msg is None:
                continue

            if msg.type == 'note_on' or msg.type == 'note_off':
                # Always send original note to first output
                output_ports_list[0].send(msg)

                # Generate harmony note based on mode
                if mode == 1:
                    # Forward as-is to all outputs
                    for port in output_ports_list[1:]:
                        port.send(msg)
                else:
                    # Generate harmony note
                    if msg.type == 'note_on' and msg.velocity > 0:
                        if mode == 2:
                            harmony_note = find_nearest_diatonic_third(msg.note, key)
                        elif mode == 3:
                            harmony_note = find_random_diatonic_below(msg.note, key)
                        elif mode == 4:
                            harmony_note = find_random_diatonic_below_no_seconds(msg.note, key)
                        elif mode == 5:
                            harmony_note = find_contrary_diatonic_below_no_seconds(
                                msg.note, key, prev_input_note, prev_output_note)
                        else:  # mode 6
                            harmony_note = find_strict_counterpoint_below(
                                msg.note, key, prev_input_note, prev_output_note)
                        
                        active_notes[msg.note] = harmony_note
                        harmony_msg = msg.copy(note=int(harmony_note))
                        # harmony_msg.velocity = scale_velocity(msg.velocity)
                        
                        # Send to all harmony outputs
                        for port in output_ports_list[1:]:
                            port.send(harmony_msg)
                            
                        if mode in [5, 6]:
                            prev_input_note = msg.note
                            prev_output_note = harmony_note
                    else:
                        if msg.note in active_notes:
                            harmony_msg = msg.copy(note=int(active_notes[msg.note]))
                            # if msg.type == 'note_on':
                                # harmony_msg.velocity = scale_velocity(msg.velocity)
                            
                            # Send to all harmony outputs
                            for port in output_ports_list[1:]:
                                port.send(harmony_msg)
                                
                            if msg.type == 'note_off' or msg.velocity == 0:
                                del active_notes[msg.note]
                                if msg.note == prev_input_note:
                                    prev_input_note = None
                                    prev_output_note = None
            else:
                # Forward all other MIDI messages to all outputs
                for port in output_ports_list:
                    port.send(msg)

    except KeyboardInterrupt:
        # Make sure to turn off any active notes before exiting
        if mode != 1:
            for input_note, generated_note in active_notes.items():
                off_msg = mido.Message('note_off', note=int(generated_note), velocity=0)
                for port in output_ports_list[1:]:
                    port.send(off_msg)
        
        print("\nExiting...")
        input_port.close()
        for port in output_ports_list:
            port.close()

if __name__ == '__main__':
    main()
