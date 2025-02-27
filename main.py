import mido
import random
import sys
import termios
import tty
import threading
import queue
import argparse
import curses
from audio_to_midi import AudioToMidi
import sounddevice as sd
import numpy as np
import time

def list_and_choose_input_type():
    """Let user choose between MIDI and audio input."""
    print("\nSelect input type:")
    print("1: MIDI Input")
    print("2: Audio Input")
    
    while True:
        try:
            choice = int(input("\nEnter choice (1 or 2): "))
            if choice in [1, 2]:
                return "midi" if choice == 1 else "audio"
        except ValueError:
            pass
        print("Please enter 1 for MIDI or 2 for Audio input.")

def list_and_choose_midi_ports():
    """List all available MIDI ports and let user choose."""
    input_ports = mido.get_input_names()
    
    print("\nAvailable MIDI Input Ports:")
    for i, port in enumerate(input_ports):
        print(f"{i}: {port}")
    
    try:
        input_choice = int(input("\nChoose input port number: "))
        return input_ports[input_choice]
    except (ValueError, IndexError) as e:
        print(f"Error choosing port: {e}")
        return None

def monitor_audio_levels(stdscr, device_id, num_channels, sample_rate):
    """Monitor audio levels using curses for display."""
    # Set up colors
    curses.start_color()
    curses.init_pair(1, curses.COLOR_GREEN, curses.COLOR_BLACK)
    curses.init_pair(2, curses.COLOR_YELLOW, curses.COLOR_BLACK)
    curses.init_pair(3, curses.COLOR_RED, curses.COLOR_BLACK)
    
    # Hide the cursor
    curses.curs_set(0)
    
    # Get device info for channel names
    device_info = sd.query_devices(device_id)
    print(f"Device info: {device_info}")  # Debug info
    
    # Get actual number of input channels from device
    actual_channels = min(device_info['max_input_channels'], num_channels)
    if actual_channels == 0:
        raise ValueError(f"No input channels available on device {device_id}")
    
    # Clear screen
    stdscr.clear()
    
    # Create the header
    stdscr.addstr(0, 0, "Raw Audio Input Monitor (Press 'q' to stop)")
    stdscr.addstr(1, 0, f"Device: {device_info['name']}")
    stdscr.addstr(2, 0, f"Sample Rate: {sample_rate}Hz, Buffer Size: 512, Channels: {actual_channels}")
    stdscr.addstr(3, 0, "Raw Input Values:")
    
    # Initialize buffers for displaying raw values
    raw_buffers = [[] for _ in range(actual_channels)]
    max_buffer_size = 100  # Keep last 100 samples for display
    
    def audio_callback(indata, frames, time, status):
        if status:
            stdscr.addstr(actual_channels + 8, 0, f"Status: {status}")
            stdscr.refresh()
            return
        
        try:
            # Get raw input data for each channel
            for i in range(actual_channels):
                # Get raw data for this channel
                raw_data = indata[:, i]
                
                # Store last few raw values
                raw_buffers[i].extend(raw_data)
                if len(raw_buffers[i]) > max_buffer_size:
                    raw_buffers[i] = raw_buffers[i][-max_buffer_size:]
                
                # Clear the line
                stdscr.addstr(i + 5, 0, " " * 120)
                
                # Show channel info
                channel_info = f"Channel {i}: "
                stdscr.addstr(i + 5, 0, channel_info)
                
                # Show raw values as a simple oscilloscope
                raw_min = min(raw_buffers[i])
                raw_max = max(raw_buffers[i])
                raw_current = raw_data[-1]
                
                # Display raw values
                value_info = f"Current: {raw_current:+.6f} Min: {raw_min:+.6f} Max: {raw_max:+.6f}"
                stdscr.addstr(i + 5, len(channel_info), value_info)
                
                # Show a simple visualization
                meter_pos = 60
                meter_width = 50
                level = int((raw_current - raw_min) / (raw_max - raw_min + 1e-10) * meter_width) if raw_max > raw_min else 0
                
                for j in range(meter_width):
                    if j < level:
                        color = curses.color_pair(1)
                        stdscr.addstr(i + 5, meter_pos + j, "█", color)
                    else:
                        stdscr.addstr(i + 5, meter_pos + j, "─")
            
            # Add debug info
            stdscr.addstr(actual_channels + 6, 0, f"Buffer size: {frames} samples")
            stdscr.addstr(actual_channels + 7, 0, f"Raw buffer stats - Shape: {indata.shape}, Type: {indata.dtype}")
            stdscr.addstr(actual_channels + 8, 0, f"Raw data range - Min: {np.min(indata):.6f}, Max: {np.max(indata):.6f}")
            
            # Refresh the screen
            stdscr.refresh()
            
        except Exception as e:
            # Add error display at the bottom of the screen
            stdscr.addstr(actual_channels + 11, 0, f"Error: {str(e)}")
            stdscr.refresh()
    
    # Start the audio stream
    try:
        with sd.InputStream(
            device=device_id,
            channels=actual_channels,
            callback=audio_callback,
            blocksize=512,
            samplerate=48000,
            dtype=np.float32
        ):
            stdscr.addstr(actual_channels + 12, 0, "Stream started successfully")
            stdscr.refresh()
            
            # Main event loop
            while True:
                c = stdscr.getch()
                if c == ord('q'):
                    break
                time.sleep(0.01)
    except Exception as e:
        stdscr.addstr(actual_channels + 13, 0, f"Stream error: {str(e)}")
        stdscr.refresh()
        time.sleep(2)

def list_and_choose_audio_devices():
    """List all available audio input devices and let user choose."""
    audio_devices = AudioToMidi.list_audio_devices()
    
    print("\nAvailable Audio Input Devices:")
    for i, name, channels in audio_devices:
        print(f"{i}: {name} ({channels} channels)")
    
    while True:
        try:
            print("\nWould you like to monitor input levels before choosing? (y/n)")
            monitor = input().lower().strip()
            if monitor == 'y':
                print("\nStarting audio monitor...")
                
                try:
                    input_choice = int(input("\nEnter device number to monitor: "))
                    # Find device info
                    device_info = None
                    for i, name, channels in audio_devices:
                        if i == input_choice:
                            device_info = sd.query_devices(i)
                            break
                    
                    if device_info is None:
                        print("Invalid device number")
                        continue
                    
                    # Start the curses-based monitor
                    curses.wrapper(
                        monitor_audio_levels,
                        input_choice,
                        device_info['max_input_channels'],
                        int(device_info['default_samplerate'])
                    )
                    
                except Exception as e:
                    print(f"Error during monitoring: {e}")
                
                print("\nDone monitoring.")
            
            print("\nChoose audio device number:")
            input_choice = int(input())
            # Verify the choice is valid
            for i, _, _ in audio_devices:
                if i == input_choice:
                    print("\nWhich channel would you like to use? (0 for first channel, 1 for second, etc.)")
                    channel = int(input())
                    # Store the selected channel in the AudioToMidi instance
                    return input_choice, channel
            raise ValueError("Invalid device number")
        except ValueError as e:
            print(f"Error: {e}")
            print("Please try again.")
        except KeyboardInterrupt:
            print("\nMonitoring stopped.")
            continue
    return None, 0

def list_and_choose_output_ports(num_outputs):
    """List all available MIDI output ports and let user choose multiple."""
    output_ports = mido.get_output_names()
    
    print("\nAvailable MIDI Output Ports:")
    for i, port in enumerate(output_ports):
        print(f"{i}: {port}")
    
    chosen_ports = []
    for i in range(num_outputs):
        while True:
            try:
                if i == 0:
                    port_num = int(input(f"\nSelect output port {i+1} (melody): "))
                else:
                    port_num = int(input(f"Select output port {i+1} (harmony): "))
                if 0 <= port_num < len(output_ports):
                    chosen_ports.append(output_ports[port_num])
                    break
                print("Invalid port number. Please try again.")
            except ValueError:
                print("Please enter a valid number.")
    
    return chosen_ports

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

def find_nearest_diatonic_fourth(note, key):
    """Find the nearest diatonic fourth below the note in the given key"""
    scale_notes = get_scale_notes(key)
    base_note = note % 12
    
    # Find the nearest scale note for the input
    if base_note not in scale_notes:
        distances = [(abs(base_note - scale_note), scale_note) for scale_note in scale_notes]
        base_note = min(distances, key=lambda x: x[0])[1]
    
    # Find the note three scale degrees below
    base_index = scale_notes.index(base_note)
    fourth_index = (base_index - 3) % 7  # Three scale degrees below
    fourth_note = scale_notes[fourth_index]
    
    # Adjust octave to be below the input note
    current_octave = note // 12
    fourth_note = fourth_note + (current_octave * 12)
    
    # If the fourth is still above or equal to the input note, move it down an octave
    if fourth_note >= note:
        fourth_note -= 12
    
    return int(fourth_note)

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
    # Parse command line arguments
    parser = argparse.ArgumentParser(description='Contrapunk - A MIDI counterpoint generator')
    parser.add_argument('--ui', action='store_true', help='Use graphical user interface instead of CLI')
    args = parser.parse_args()

    if args.ui:
        from contrapunk_ui import run_ui
        run_ui()
        return

    # Choose input type (MIDI or Audio)
    input_type = list_and_choose_input_type()
    
    # Set up input based on type
    input_source = None
    if input_type == "midi":
        input_port_name = list_and_choose_midi_ports()
        if input_port_name is None:
            print("Failed to select MIDI input port.")
            return
        input_source = mido.open_input(input_port_name)
    else:  # audio
        device_id, channel = list_and_choose_audio_devices()
        if device_id is None:
            print("Failed to select audio input device.")
            return
        input_source = AudioToMidi(device_id=device_id, input_channel=channel)
        input_source.start()

    # Select number of outputs
    print("\nHow many outputs do you want? (minimum 2)")
    try:
        num_outputs = max(2, int(input("Enter number of outputs: ")))
    except ValueError:
        print("Invalid input. Using default of 2 outputs.")
        num_outputs = 2

    # Select output ports
    output_ports = list_and_choose_output_ports(num_outputs)
    output_ports_list = [mido.open_output(port) for port in output_ports]

    # Select key
    key_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    print("\nSelect key:")
    for i, key in enumerate(key_names):
        print(f"{i}: {key}")
    try:
        key = int(input("Enter key number: "))
    except ValueError:
        print("Invalid input. Using C major (key 0).")
        key = 0

    # Select initial mode
    print("\nSelect mode:")
    print("1: Forward MIDI as-is")
    print("2: Add diatonic thirds")
    print("3: Add diatonic fourths")
    print("4: Add random diatonic intervals")
    print("5: Add random diatonic intervals (no seconds)")
    print("6: Add contrary motion with random intervals (no seconds)")
    print("7: Add strict counterpoint rules")
    print("\nYou can change modes during runtime using number keys 1-7")
    print("Press 'q' to quit")
    try:
        mode = int(input("Enter mode number: "))
    except ValueError:
        print("Invalid input. Using mode 1.")
        mode = 1

    if input_type == "midi":
        print(f"\nInput: {input_port_name}")
    else:
        print(f"\nInput: Audio Device {device_id}")
    print("Outputs:")
    for i, port in enumerate(output_ports):
        print(f"Output {i+1}: {port}")
    print("\nListening for input...")
    print("Current mode:", mode)
    print("Press 1-7 to change modes, 'q' to quit")

    scale_notes = get_scale_notes(key)
    active_notes = {}  # Maps input note to list of generated notes
    prev_input_notes = {}  # Track previous input notes for each voice
    prev_output_notes = {}  # Track previous output notes for each voice
    
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
                    prev_input_notes = {}
                    prev_output_notes = {}
                    print(f"\nChanged to mode {mode}")
                elif command == 'quit':
                    raise KeyboardInterrupt
            except queue.Empty:
                pass

            # Get MIDI message from either MIDI input or audio converter
            msg = None
            if input_type == "midi":
                msg = input_source.poll()
            else:
                msg = input_source.get_midi_message()

            if msg is None:
                continue

            # Rest of the MIDI processing remains the same
            if msg.type == 'note_on' or msg.type == 'note_off':
                # Always send original note to first output
                output_ports_list[0].send(msg)

                # Generate harmony notes based on mode
                if mode == 1:
                    # Forward as-is to all outputs
                    for port in output_ports_list[1:]:
                        port.send(msg)
                else:
                    # Generate harmony notes
                    if msg.type == 'note_on' and msg.velocity > 0:
                        # Initialize list of harmony notes for this input
                        harmony_notes = []
                        
                        # Generate first harmony from the input note
                        prev_in = prev_input_notes.get(1)
                        prev_out = prev_output_notes.get(1)
                        source_note = msg.note
                        
                        # Generate each harmony voice
                        for voice in range(len(output_ports_list) - 1):
                            if mode == 2:
                                harmony_note = find_nearest_diatonic_third(source_note, key)
                            elif mode == 3:
                                harmony_note = find_nearest_diatonic_fourth(source_note, key)
                            elif mode == 4:
                                harmony_note = find_random_diatonic_below(source_note, key)
                            elif mode == 5:
                                harmony_note = find_random_diatonic_below_no_seconds(source_note, key)
                            elif mode == 6:
                                harmony_note = find_contrary_diatonic_below_no_seconds(
                                    source_note, key, prev_in, prev_out)
                            else:  # mode 7
                                harmony_note = find_strict_counterpoint_below(
                                    source_note, key, prev_in, prev_out)
                            
                            harmony_notes.append(harmony_note)
                            harmony_msg = msg.copy(note=int(harmony_note))
                            output_ports_list[voice + 1].send(harmony_msg)
                            
                            # Update tracking for next voice
                            if mode in [5, 6]:
                                prev_input_notes[voice + 1] = source_note
                                prev_output_notes[voice + 1] = harmony_note
                            
                            # Next harmony will be generated from this one
                            source_note = harmony_note
                            prev_in = prev_input_notes.get(voice + 1)
                            prev_out = prev_output_notes.get(voice + 1)
                        
                        active_notes[msg.note] = harmony_notes
                    else:
                        if msg.note in active_notes:
                            harmony_notes = active_notes[msg.note]
                            for voice, harmony_note in enumerate(harmony_notes):
                                harmony_msg = msg.copy(note=int(harmony_note))
                                output_ports_list[voice + 1].send(harmony_msg)
                            
                            if msg.type == 'note_off' or msg.velocity == 0:
                                del active_notes[msg.note]
                                if msg.note in prev_input_notes.values():
                                    # Clear motion tracking for all affected voices
                                    for voice in range(len(output_ports_list) - 1):
                                        if prev_input_notes.get(voice + 1) == msg.note:
                                            prev_input_notes[voice + 1] = None
                                            prev_output_notes[voice + 1] = None
            else:
                # Forward all other MIDI messages to all outputs
                for port in output_ports_list:
                    port.send(msg)

    except KeyboardInterrupt:
        # Make sure to turn off any active notes before exiting
        if mode != 1:
            for input_note, generated_notes in active_notes.items():
                for harmony_note in generated_notes:
                    off_msg = mido.Message('note_off', note=int(harmony_note), velocity=0)
                    for port in output_ports_list[1:]:
                        port.send(off_msg)
        
        print("\nExiting...")
        if input_type == "midi":
            input_source.close()
        else:
            input_source.stop()
        for port in output_ports_list:
            port.close()

if __name__ == '__main__':
    main()
