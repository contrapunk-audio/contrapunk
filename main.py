import mido
import random

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
    print("3: Add random diatonic intervals")
    mode = int(input("Enter mode number: "))

    # Open output ports
    output_port1 = mido.open_output(output_ports[output1_num])
    output_port2 = mido.open_output(output_ports[output2_num])

    print(f"\nInput: {available_ports[port_num]}")
    print(f"Output 1: {output_ports[output1_num]}")
    print(f"Output 2: {output_ports[output2_num]}")
    print("\nListening for MIDI messages... Press Ctrl+C to exit.")

    scale_notes = get_scale_notes(key) if mode in [2, 3] else None
    
    # Dictionary to track active notes for mode 3
    active_notes = {}  # Maps input note to generated note

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
                elif mode == 3:
                    # Handle random diatonic intervals with proper note tracking
                    if msg.type == 'note_on' and msg.velocity > 0:
                        # Generate new random note for note-on
                        random_note = find_random_diatonic_below(msg.note, key)
                        active_notes[msg.note] = random_note
                        random_msg = msg.copy(note=int(random_note))
                        output_port2.send(random_msg)
                    else:
                        # For note-off or zero velocity note-on, use the stored note
                        if msg.note in active_notes:
                            random_msg = msg.copy(note=int(active_notes[msg.note]))
                            output_port2.send(random_msg)
                            if msg.type == 'note_off' or msg.velocity == 0:
                                del active_notes[msg.note]  # Clean up note tracking
            else:
                # Forward all other MIDI messages to both outputs
                output_port1.send(msg)
                output_port2.send(msg)

    except KeyboardInterrupt:
        # Make sure to turn off any active notes before exiting
        if mode == 3:
            for input_note, generated_note in active_notes.items():
                off_msg = mido.Message('note_off', note=int(generated_note), velocity=0)
                output_port2.send(off_msg)
        
        print("\nExiting...")
        input_port.close()
        output_port1.close()
        output_port2.close()

if __name__ == '__main__':
    main()
