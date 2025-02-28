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
import contextlib
import select
import os

# Musical constants
NOTES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
BASE_NOTE = 60  # Middle C

# Define chord progressions
CHORD_PROGRESSIONS = {
    'I-IV-V-I': [(0, 'major'), (5, 'major'), (7, 'major'), (0, 'major')],
    'ii-V-I': [(2, 'minor'), (7, 'major'), (0, 'major')],
    'I-vi-ii-V': [(0, 'major'), (9, 'minor'), (2, 'minor'), (7, 'major')],
    # J-Pop Progressions
    'Royal Road (I-V-vi-iii-IV-I-IV-V)': [(0, 'major'), (7, 'major'), (9, 'minor'), (4, 'minor'), (5, 'major'), (0, 'major'), (5, 'major'), (7, 'major')],
    'Marusa (I-vi-IV-V)': [(0, 'major'), (9, 'minor'), (5, 'major'), (7, 'major')],
    # Additional Progressions
    "Pachelbel's Canon (I-V-vi-iii-IV-I-IV-V)": [(0, 'major'), (7, 'major'), (9, 'minor'), (4, 'minor'), (5, 'major'), (0, 'major'), (5, 'major'), (7, 'major')],
    'Axis of Awesome (I-V-vi-IV)': [(0, 'major'), (7, 'major'), (9, 'minor'), (5, 'major')],
    "50s Progression (I-vi-IV-V)": [(0, 'major'), (9, 'minor'), (5, 'major'), (7, 'major')],
    'Pop Punk Progression (I-V-vi-IV)': [(0, 'major'), (7, 'major'), (9, 'minor'), (5, 'major')],
    'Jazz Progression (ii-V-I)': [(2, 'minor'), (7, 'major'), (0, 'major')],
    'Sakura (I-IV-V-vi)': [(0, 'major'), (5, 'major'), (7, 'major'), (9, 'minor')],
    'Ballad (vi-IV-I-V)': [(9, 'minor'), (5, 'major'), (0, 'major'), (7, 'major')],
    'Nostalgia (IV-V-iii-vi)': [(5, 'major'), (7, 'major'), (4, 'minor'), (9, 'minor')],
    'Adventure (I-V-vi-iii-IV-I-IV-V)': [(0, 'major'), (7, 'major'), (9, 'minor'), (4, 'minor'), (5, 'major'), (0, 'major'), (5, 'major'), (7, 'major')],
    'Dream (I-vi-ii-IV)': [(0, 'major'), (9, 'minor'), (2, 'minor'), (5, 'major')],
}

# Define popular rhythm patterns
RHYTHM_PATTERNS = {
    'Simple 4/4 (Four Quarter Notes)': [1.0, 1.0, 1.0, 1.0],
    'March (Steady Half Notes)': [0.5, 0.5, 0.5, 0.5],
    'Waltz (3/4 Time)': [0.75, 0.25, 0.75, 0.25],
    'Swing Jazz': [0.5, 0.25, 0.25, 0.5],
    'Latin Groove': [0.5, 0.5, 0.25, 0.25, 0.5],
    'Syncopated Pop': [0.25, 0.5, 0.25, 0.5],
    'Classical Dotted': [0.75, 0.25, 1.0],
    'Folk Ballad': [1.0, 0.5, 0.5],
    'Rock Backbeat': [0.5, 0.25, 0.5, 0.25]
}

class ContrapunkTUI:
    """Text User Interface for Contrapunk using curses."""
    def __init__(self, stdscr):
        self.stdscr = stdscr
        self.setup_colors()
        self.current_menu = []
        self.selected_index = 0
        self.max_y, self.max_x = stdscr.getmaxyx()
        self.status_message = ""
        self.title = ""
        self.active_notes = {}  # Dictionary to track active notes and their harmonies
        
    def setup_colors(self):
        """Initialize color pairs."""
        curses.start_color()
        curses.init_pair(1, curses.COLOR_GREEN, curses.COLOR_BLACK)  # Selected item
        curses.init_pair(2, curses.COLOR_CYAN, curses.COLOR_BLACK)   # Title
        curses.init_pair(3, curses.COLOR_YELLOW, curses.COLOR_BLACK) # Status
        curses.init_pair(4, curses.COLOR_RED, curses.COLOR_BLACK)    # Error
        curses.init_pair(5, curses.COLOR_WHITE, curses.COLOR_BLACK)  # Normal text
        curses.init_pair(6, curses.COLOR_MAGENTA, curses.COLOR_BLACK) # Audio levels
        
    def draw_title(self):
        """Draw the title at the top of the screen."""
        title = f" {self.title} "
        x = max(0, (self.max_x - len(title)) // 2)
        self.stdscr.attron(curses.color_pair(2) | curses.A_BOLD)
        self.stdscr.addstr(0, x, title)
        self.stdscr.attroff(curses.color_pair(2) | curses.A_BOLD)
        self.stdscr.hline(1, 0, curses.ACS_HLINE, self.max_x)
        
    def draw_status(self):
        """Draw the status message at the bottom of the screen."""
        self.stdscr.hline(self.max_y-2, 0, curses.ACS_HLINE, self.max_x)
        self.stdscr.attron(curses.color_pair(3))
        self.stdscr.addstr(self.max_y-1, 0, self.status_message[:self.max_x-1])
        self.stdscr.attroff(curses.color_pair(3))
        
    def draw_menu(self):
        """Draw the current menu items."""
        start_y = 3
        for i, item in enumerate(self.current_menu):
            if i >= self.max_y - 4:  # Leave space for title and status
                break
                
            # Highlight selected item
            if i == self.selected_index:
                self.stdscr.attron(curses.color_pair(1) | curses.A_BOLD)
                
            # Center the menu item
            item_str = f" {item} "
            x = max(0, (self.max_x - len(item_str)) // 2)
            self.stdscr.addstr(start_y + i, x, item_str)
            
            if i == self.selected_index:
                self.stdscr.attroff(curses.color_pair(1) | curses.A_BOLD)
                
    def update_screen(self):
        """Update the entire screen."""
        self.stdscr.clear()
        self.draw_title()
        self.draw_menu()
        self.draw_status()
        self.stdscr.refresh()
        
    def show_menu(self, title, items, status="Use ↑↓ to select, Enter to confirm"):
        """Show a menu and return the selected index."""
        self.title = title
        self.current_menu = items
        self.status_message = status
        self.selected_index = 0
        
        while True:
            self.update_screen()
            key = self.stdscr.getch()
            
            if key == curses.KEY_UP and self.selected_index > 0:
                self.selected_index -= 1
            elif key == curses.KEY_DOWN and self.selected_index < len(items) - 1:
                self.selected_index += 1
            elif key == ord('\n'):  # Enter key
                return self.selected_index
            elif key == ord('q'):
                return -1
                
    def show_value_input(self, prompt, min_val=None, max_val=None):
        """Show an input prompt for numeric values."""
        self.status_message = prompt
        value = ""
        while True:
            self.update_screen()
            
            # Show current input
            x = max(0, (self.max_x - len(prompt) - len(value) - 3) // 2)
            self.stdscr.addstr(self.max_y // 2, x, f"{prompt}: {value}")
            
            key = self.stdscr.getch()
            if key == ord('\n') and value:  # Enter key
                try:
                    val = int(value)
                    if (min_val is None or val >= min_val) and (max_val is None or val <= max_val):
                        return val
                    else:
                        self.show_error(f"Value must be between {min_val} and {max_val}")
                except ValueError:
                    self.show_error("Please enter a valid number")
            elif key == curses.KEY_BACKSPACE or key == 127:  # Backspace
                value = value[:-1]
            elif 48 <= key <= 57:  # Numbers 0-9
                value += chr(key)
            elif key == ord('q'):
                return None
                
    def show_error(self, message, delay=1):
        """Show an error message briefly."""
        self.stdscr.attron(curses.color_pair(4) | curses.A_BOLD)
        x = max(0, (self.max_x - len(message)) // 2)
        self.stdscr.addstr(self.max_y // 2 + 2, x, message)
        self.stdscr.attroff(curses.color_pair(4) | curses.A_BOLD)
        self.stdscr.refresh()
        curses.napms(delay * 1000)
        
    def run_audio_monitor(self, device_id, num_channels, sample_rate):
        """Run the audio level monitor interface."""
        self.title = "Audio Monitor"
        self.status_message = "Press 'q' to stop monitoring"
        
        # Initialize audio buffers
        raw_buffers = [[] for _ in range(num_channels)]
        max_buffer_size = 100
        
        def audio_callback(indata, frames, time, status):
            if status:
                self.show_error(f"Status: {status}")
                return
                
            try:
                for i in range(num_channels):
                    raw_data = indata[:, i]
                    raw_buffers[i].extend(raw_data)
                    if len(raw_buffers[i]) > max_buffer_size:
                        raw_buffers[i] = raw_buffers[i][-max_buffer_size:]
                    
                    # Clear line
                    self.stdscr.addstr(i + 5, 0, " " * self.max_x)
                    
                    # Show channel info
                    raw_min = min(raw_buffers[i])
                    raw_max = max(raw_buffers[i])
                    raw_current = raw_data[-1]
                    
                    info = f"Channel {i}: {raw_current:+.3f} [{raw_min:+.3f} to {raw_max:+.3f}]"
                    self.stdscr.addstr(i + 5, 2, info)
                    
                    # Draw level meter
                    meter_width = self.max_x - len(info) - 6
                    level = int((raw_current - raw_min) / (raw_max - raw_min + 1e-10) * meter_width)
                    
                    self.stdscr.attron(curses.color_pair(6))
                    self.stdscr.addstr(i + 5, len(info) + 4, "█" * level + "░" * (meter_width - level))
                    self.stdscr.attroff(curses.color_pair(6))
                
                self.stdscr.refresh()
                
            except Exception as e:
                self.show_error(f"Error: {e}")
        
        try:
            with sd.InputStream(
                device=device_id,
                channels=num_channels,
                callback=audio_callback,
                blocksize=512,
                samplerate=sample_rate,
                dtype=np.float32
            ):
                while True:
                    key = self.stdscr.getch()
                    if key == ord('q'):
                        break
                    time.sleep(0.01)
        except Exception as e:
            self.show_error(f"Stream error: {e}")
            time.sleep(2)
            
    def run_mode_based_processor(self, key, mode, command_queue):
        """Run the mode-based MIDI processor interface."""
        self.title = "Mode-based MIDI Processor"
        self.status_message = "Press 1-7 to change modes, 'q' to quit"
        
        mode_names = [
            "Forward MIDI as-is",
            "Add diatonic thirds",
            "Add diatonic fourths",
            "Add random diatonic intervals",
            "Add random diatonic intervals (no seconds)",
            "Add contrary motion",
            "Strict counterpoint"
        ]
        
        while True:
            self.current_menu = [
                f"Current Key: {NOTES[key]}",
                f"Current Mode: {mode_names[mode-1]}",
                "",
                "Active Notes:",
                *[f"Note {note}: {', '.join(map(str, harmonies))}" 
                  for note, harmonies in self.active_notes.items()],
                "",
                "Press 1-7 to change modes",
                "Press 'k' to change key",
                "Press 'q' to quit"
            ]
            
            self.update_screen()
            
            if not command_queue.empty():
                cmd = command_queue.get()
                if cmd.isdigit() and 1 <= int(cmd) <= 7:
                    return ('m', int(cmd))
                elif cmd == 'k':
                    return ('k', None)
                elif cmd == 'q':
                    return ('q', None)
            
            time.sleep(0.1)
            
    def run_midi_processor(self, input_port, output_ports):
        """Run the MIDI processor interface."""
        self.title = "MIDI Processor"
        self.status_message = "Press 'q' to quit"
        
        while True:
            # Process MIDI messages
            msg = input_port.poll()
            if msg:
                # Display MIDI activity
                if msg.type in ['note_on', 'note_off']:
                    self.current_menu = [
                        f"Input Port: {input_port}",
                        f"Output Ports: {', '.join(output_ports)}",
                        "",
                        f"Last Message: {msg.type} note={msg.note} velocity={msg.velocity}"
                    ]
                else:
                    self.current_menu = [
                        f"Input Port: {input_port}",
                        f"Output Ports: {', '.join(output_ports)}",
                        "",
                        f"Last Message: {msg.type}"
                    ]
                self.update_screen()
            
            # Check for quit
            if self.stdscr.getch() == ord('q'):
                break
                
    def run_music_player(self, key, progression, harmony_modes):
        """Show the music player interface."""
        self.title = "Music Player"
        self.status_message = "Press 'q' to quit, 'n' for new melody, 'k' to change key"
        
        mode_names = [
            "Forward",
            "Thirds",
            "Fourths",
            "Random",
            "No Seconds",
            "Contrary",
            "Counterpoint"
        ]
        
        while True:
            self.current_menu = [
                f"Current Key: {NOTES[key]}",
                f"Current Progression: {progression}",
                "Harmony Modes:",
                *[f"Voice {i+1}: {mode_names[mode-1]}" 
                  for i, mode in enumerate(harmony_modes)],
                "",
                "Playing..."
            ]
            
            self.update_screen()
            key = self.stdscr.getch()
            
            if key == ord('q'):
                return 'q'
            elif key == ord('n'):
                return 'n'
            elif key == ord('k'):
                return 'k'

def clear_terminal():
    """Clear the terminal screen in a cross-platform way."""
    if sys.platform.startswith('win'):
        os.system('cls')
    else:
        os.system('clear')

def print_status(message, clear=False):
    """Print status messages in a consistent way.
    If clear is True, clear the line before printing."""
    if clear:
        # Move cursor to beginning of line and clear line
        sys.stdout.write('\r\033[K')
    sys.stdout.write(message)
    sys.stdout.flush()

def print_menu(title, options, show_numbers=True):
    """Print a menu with options in a consistent format."""
    print(f"\n{title}")
    print("=" * len(title))
    for i, option in enumerate(options):
        if show_numbers:
            print(f"{i+1}: {option}")
        else:
            print(option)
    print()

def generate_melody(key, length=16, tempo=120, progression='I-IV-V-I', rhythm_pattern=[1.0, 1.0, 1.0, 1.0]):
    """Generate a melodic sequence in the given key following a chord progression and rhythm pattern."""
    scale = get_scale_notes(key)
    melody = []
    current_note = random.choice(scale) + BASE_NOTE
    
    # Get the chord progression
    chords = CHORD_PROGRESSIONS[progression]
    
    for i in range(length):
        # Use the selected rhythm pattern for each measure
        rhythm_index = 0
        
        # Determine the current chord
        chord_root, chord_type = chords[i % len(chords)]
        chord_notes = get_chord_notes(chord_root, chord_type, key)
        
        # Determine the next note movement
        step = random.choice([-2, -1, 0, 1, 2])  # Allow steps within a reasonable range
        scale_pos = scale.index(current_note % 12)
        new_scale_pos = (scale_pos + step) % len(scale)
        new_note = chord_notes[new_scale_pos % len(chord_notes)] + BASE_NOTE + ((current_note - BASE_NOTE) // 12) * 12
        
        # Adjust octave if necessary
        if new_note < BASE_NOTE:
            new_note += 12
        elif new_note > BASE_NOTE + 24:
            new_note -= 12
            
        # Create note event with duration from rhythm pattern
        duration = rhythm_pattern[rhythm_index % len(rhythm_pattern)]
        rhythm_index += 1
        velocity = random.randint(64, 96)  # Varying velocity for dynamics
        melody.append((new_note, duration, velocity))
        current_note = new_note
        
    return melody

def generate_harmony(melody, key, voice_number=1, harmony_mode=7):
    """Generate harmony for a given melody using selected harmony mode.
    harmony_mode options:
    1: Forward melody as-is
    2: Add diatonic thirds
    3: Add diatonic fourths
    4: Add random diatonic intervals
    5: Add random diatonic intervals (no seconds)
    6: Add contrary motion with random intervals (no seconds)
    7: Add strict counterpoint rules (default)
    """
    harmony = []
    prev_melody = None
    prev_harmony = None
    
    for note, duration, velocity in melody:
        # Generate harmony based on selected mode
        if harmony_mode == 1:
            # Forward melody as-is
            harmony_note = note
        elif harmony_mode == 2:
            harmony_note = find_nearest_diatonic_third(note, key)
        elif harmony_mode == 3:
            harmony_note = find_nearest_diatonic_fourth(note, key)
        elif harmony_mode == 4:
            harmony_note = find_random_diatonic_below(note, key)
        elif harmony_mode == 5:
            harmony_note = find_random_diatonic_below_no_seconds(note, key)
        elif harmony_mode == 6:
            harmony_note = find_contrary_diatonic_below_no_seconds(note, key, prev_melody, prev_harmony)
        else:  # mode 7 - strict counterpoint
            # Different voice numbers get different preferred intervals
            if voice_number == 1:
                harmony_note = find_strict_counterpoint_below(note, key, prev_melody, prev_harmony)
            elif voice_number == 2:
                harmony_note = find_strict_counterpoint_below(note, key, prev_melody, prev_harmony)
            else:
                harmony_note = find_strict_counterpoint_below(note, key, prev_melody, prev_harmony)
        
        # Reduce velocity more for each subsequent voice
        harmony.append((harmony_note, duration, velocity - (5 * voice_number)))
        prev_melody = note
        prev_harmony = harmony_note
        
    return harmony

def play_generated_music(output_ports, command_queue, tui, key=0, tempo=120, progression='I-IV-V-I', rhythm_pattern=[1.0, 1.0, 1.0, 1.0], harmony_modes=None):
    """Play generated music through multiple MIDI output ports following a chord progression."""
    try:
        # Open all output ports
        with contextlib.ExitStack() as stack:
            ports = [stack.enter_context(mido.open_output(port)) for port in output_ports]
            
            while True:
                # Generate melody and harmonies
                tui.draw_status("Generating new melody...")
                melody = generate_melody(key, length=16, tempo=tempo, progression=progression, rhythm_pattern=rhythm_pattern)
                harmonies = [
                    generate_harmony(melody, key, voice_number=i+1, harmony_mode=harmony_modes[i] if harmony_modes else 7)
                    for i in range(len(output_ports)-1)
                ]
                tui.draw_status("Playing...")
                
                # Play all parts simultaneously
                for step in range(len(melody)):
                    # Check for user input
                    if not command_queue.empty():
                        cmd = command_queue.get()
                        if cmd == 'q':
                            return
                        elif cmd == 'n':
                            tui.draw_status("Generating new melody...")
                            break
                        elif cmd == 'k':
                            key = (key + 1) % 12
                            tui.draw_status(f"Changed key to: {NOTES[key]}")
                            break
                    
                    # Get the note data for this step
                    m_note, duration, m_vel = melody[step]
                    
                    # Track all active notes for proper cleanup
                    active_notes = []
                    
                    # Send melody note to first port (if available)
                    if ports and len(ports) > 0:
                        try:
                            msg = mido.Message('note_on', note=m_note, velocity=m_vel)
                            ports[0].send(msg)
                            active_notes.append((0, m_note))  # Track (port_index, note)
                            
                            # Update TUI display for melody note
                            if m_note not in tui.active_notes:
                                tui.active_notes[m_note] = []
                            tui.active_notes[m_note].append('melody')
                            
                        except Exception as e:
                            tui.show_error(f"Error sending melody note: {str(e)}")
                            continue
                    
                    # Send harmony notes to additional ports
                    for voice, harmony in enumerate(harmonies):
                        if voice + 1 < len(ports):  # Check if port exists
                            try:
                                h_note, _, h_vel = harmony[step]
                                msg = mido.Message('note_on', note=h_note, velocity=h_vel)
                                ports[voice + 1].send(msg)
                                active_notes.append((voice + 1, h_note))
                                
                                # Update TUI display for harmony note
                                if h_note not in tui.active_notes:
                                    tui.active_notes[h_note] = []
                                tui.active_notes[h_note].append(f'harmony{voice+1}')
                                
                            except Exception as e:
                                tui.show_error(f"Error sending harmony note to voice {voice + 1}: {str(e)}")
                                continue
                    
                    # Wait for note duration
                    time.sleep(duration * 60 / tempo)
                    
                    # Send note-off messages for all active notes
                    for port_idx, note in active_notes:
                        try:
                            if port_idx < len(ports):
                                msg = mido.Message('note_off', note=note, velocity=0)
                                ports[port_idx].send(msg)
                                
                                # Update TUI display
                                if note in tui.active_notes:
                                    del tui.active_notes[note]
                                    
                        except Exception as e:
                            tui.show_error(f"Error sending note-off to port {port_idx}: {str(e)}")
                            continue
                    
                    # Update the TUI display
                    tui.update_screen()
                    
    except KeyboardInterrupt:
        tui.draw_status("Playback interrupted by user.")
    except Exception as e:
        tui.show_error(f"Error during playback: {str(e)}")
    finally:
        tui.draw_status("Stopping playback and closing ports...")
        # Ensure all ports are closed properly
        for port in ports:
            try:
                port.close()
            except Exception as e:
                tui.show_error(f"Error closing port: {str(e)}")

def list_and_choose_input_type():
    """Let user choose between MIDI, audio input, generated music, or mode-based processing."""
    print("\nSelect input type:")
    print("1: MIDI Input")
    print("2: Audio Input")
    print("3: Generated Music")
    print("4: Mode-based MIDI Processing")
    
    while True:
        try:
            choice = int(input("\nEnter choice (1-4): "))
            if choice in [1, 2, 3, 4]:
                if choice == 1:
                    return "midi"
                elif choice == 2:
                    return "audio"
                elif choice == 3:
                    return "generated"
                else:
                    return "mode-based"
        except ValueError:
            pass
        print("Please enter 1 for MIDI, 2 for Audio input, 3 for Generated Music, or 4 for Mode-based Processing.")

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
    """Find the nearest diatonic third above the given note in the given key."""
    scale_notes = get_scale_notes(key)
    note_in_scale = note % 12
    octave = note // 12
    
    # Find position in scale
    try:
        scale_pos = scale_notes.index(note_in_scale)
    except ValueError:
        # If note is not in scale, find nearest scale note
        nearest_pos = min(range(len(scale_notes)), 
                         key=lambda i: abs(scale_notes[i] - note_in_scale))
        scale_pos = nearest_pos
    
    # Get note two scale positions up (diatonic third)
    third_pos = (scale_pos + 2) % len(scale_notes)
    third = scale_notes[third_pos]
    
    # Adjust octave if necessary
    if third < note_in_scale:
        octave += 1
    
    return third + (octave * 12)

def find_nearest_diatonic_fourth(note, key):
    """Find the nearest diatonic fourth above the given note in the given key."""
    scale_notes = get_scale_notes(key)
    note_in_scale = note % 12
    octave = note // 12
    
    # Find position in scale
    try:
        scale_pos = scale_notes.index(note_in_scale)
    except ValueError:
        # If note is not in scale, find nearest scale note
        nearest_pos = min(range(len(scale_notes)), 
                         key=lambda i: abs(scale_notes[i] - note_in_scale))
        scale_pos = nearest_pos
    
    # Get note three scale positions up (diatonic fourth)
    fourth_pos = (scale_pos + 3) % len(scale_notes)
    fourth = scale_notes[fourth_pos]
    
    # Adjust octave if necessary
    if fourth < note_in_scale:
        octave += 1
    
    return fourth + (octave * 12)

def find_random_diatonic_below(note, key):
    """Find a random diatonic note below the given note in the given key."""
    scale_notes = get_scale_notes(key)
    note_in_scale = note % 12
    current_octave = note // 12
    
    # Get all possible notes in the scale within one octave below
    possible_notes = []
    for scale_note in scale_notes:
        if scale_note < note_in_scale:
            possible_notes.append(scale_note + (current_octave * 12))
        else:
            possible_notes.append(scale_note + ((current_octave - 1) * 12))
    
    return random.choice(possible_notes)

def find_random_diatonic_below_no_seconds(note, key):
    """Find a random diatonic note below the given note, avoiding seconds."""
    scale_notes = get_scale_notes(key)
    note_in_scale = note % 12
    current_octave = note // 12
    
    # Get all possible notes in the scale within one octave below
    possible_notes = []
    for scale_note in scale_notes:
        interval = (note_in_scale - scale_note) % 12
        if interval not in [1, 2, 10, 11]:  # Avoid seconds and sevenths
            if scale_note < note_in_scale:
                possible_notes.append(scale_note + (current_octave * 12))
            else:
                possible_notes.append(scale_note + ((current_octave - 1) * 12))
    
    return random.choice(possible_notes)

def find_contrary_diatonic_below_no_seconds(note, key, prev_input=None, prev_output=None):
    """Find a diatonic note below the given note using contrary motion and avoiding seconds."""
    scale_notes = get_scale_notes(key)
    note_in_scale = note % 12
    current_octave = note // 12
    
    # If no previous notes, use random selection
    if prev_input is None or prev_output is None:
        return find_random_diatonic_below_no_seconds(note, key)
    
    # Determine direction of input motion
    input_direction = 1 if note > prev_input else -1 if note < prev_input else 0
    
    # Get all possible notes in contrary motion
    possible_notes = []
    for scale_note in scale_notes:
        interval = (note_in_scale - scale_note) % 12
        if interval not in [1, 2, 10, 11]:  # Avoid seconds and sevenths
            if scale_note < note_in_scale:
                new_note = scale_note + (current_octave * 12)
            else:
                new_note = scale_note + ((current_octave - 1) * 12)
            
            # Check if motion is contrary
            if input_direction != 0:
                output_direction = 1 if new_note > prev_output else -1 if new_note < prev_output else 0
                if output_direction * input_direction >= 0:  # Skip if not contrary
                    continue
            
            possible_notes.append(new_note)
    
    # If no contrary motion notes found, fall back to any valid note
    if not possible_notes:
        return find_random_diatonic_below_no_seconds(note, key)
        
    return random.choice(possible_notes)

def find_strict_counterpoint_below(note, key, prev_input=None, prev_output=None, preferred_intervals=None):
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
        consonant_intervals = preferred_intervals if preferred_intervals else [3, 4, 7, 8, 9]
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
        # Add a timeout to prevent blocking indefinitely
        rlist, _, _ = select.select([sys.stdin], [], [], 0.1)
        if rlist:
            ch = sys.stdin.read(1)
        else:
            ch = None
    except Exception as e:
        print(f"\nError reading keyboard input: {e}")
        ch = None
    finally:
        # Always restore terminal settings
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
    return ch

def keyboard_input_thread(command_queue, stop_event):
    """Thread function to handle keyboard input."""
    while not stop_event.is_set():
        try:
            key = get_key_nonblocking()
            if key:
                if key in ['1', '2', '3', '4', '5', '6', '7', 'q', 'n', 'k']:
                    command_queue.put(key)
                    if key == 'q':
                        break
        except Exception as e:
            print(f"\nKeyboard thread error: {e}")
            break
    # Ensure terminal is in a good state when exiting
    try:
        termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, termios.tcgetattr(sys.stdin.fileno()))
    except:
        pass

def get_chord_notes(root, chord_type, key):
    """Return the notes of a chord based on its root and type."""
    scale_notes = get_scale_notes(key)
    root_note = (root + key) % 12
    
    if chord_type == 'major':
        intervals = [0, 4, 7]  # Major triad
    elif chord_type == 'minor':
        intervals = [0, 3, 7]  # Minor triad
    else:
        intervals = [0, 4, 7]  # Default to major triad if type is unknown
    
    chord_notes = [(root_note + interval) % 12 for interval in intervals]
    return chord_notes

def process_mode_based_midi(input_port, output_ports_list, key, command_queue, stop_event):
    """Process MIDI input using the mode-based system."""
    print("\nSelect initial mode:")
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
        mode = int(input("Enter mode number (1-7): "))
    except ValueError:
        mode = 1
    
    scale_notes = get_scale_notes(key)
    active_notes = {}  # Maps input note to list of generated notes
    prev_input_notes = {}  # Track previous input notes for each voice
    prev_output_notes = {}  # Track previous output notes for each voice
    
    print(f"\nCurrent mode: {mode}")
    print("Press 1-7 to change modes, 'q' to quit")
    
    try:
        while not stop_event.is_set():
            # Check for mode change commands
            if not command_queue.empty():
                cmd = command_queue.get()
                if cmd.isdigit() and 1 <= int(cmd) <= 7:
                    mode = int(cmd)
                    # Reset motion tracking when changing modes
                    prev_input_notes = {}
                    prev_output_notes = {}
                    print(f"\nChanged to mode {mode}")
                elif cmd == 'q':
                    break
            
            # Handle MIDI messages
            msg = input_port.poll()
            if msg is None:
                continue
            
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
        print("\nStopping mode-based processing...")
    finally:
        # Turn off any active notes
        if mode != 1:
            for input_note, generated_notes in active_notes.items():
                for harmony_note in generated_notes:
                    off_msg = mido.Message('note_off', note=int(harmony_note), velocity=0)
                    for port in output_ports_list[1:]:
                        port.send(off_msg)

def curses_main(stdscr):
    """Main function for the curses interface."""
    # Set up curses
    curses.curs_set(0)  # Hide cursor
    stdscr.nodelay(0)   # Blocking input
    stdscr.timeout(-1)  # Wait indefinitely for input
    
    # Create TUI instance
    tui = ContrapunkTUI(stdscr)
    
    # Main menu
    input_types = ["MIDI Input", "Audio Input", "Generated Music", "Mode-based MIDI Processing"]
    choice = tui.show_menu("Contrapunk", input_types)
    if choice < 0:
        return
        
    if choice == 0:  # MIDI Input
        # Select MIDI ports
        available_ports = mido.get_input_names()
        if not available_ports:
            tui.show_error("No MIDI input ports available")
            return
            
        port_choice = tui.show_menu("Select MIDI Input Port", available_ports)
        if port_choice < 0:
            return
            
        input_port = available_ports[port_choice]
        
        # Select output ports
        num_ports = tui.show_value_input("Enter number of output ports", min_val=1, max_val=8)
        if num_ports is None:
            return
            
        output_ports = []
        available_ports = mido.get_output_names()
        for i in range(num_ports):
            port_choice = tui.show_menu(f"Select Output Port {i+1}", available_ports)
            if port_choice < 0:
                return
            output_ports.append(available_ports[port_choice])
            
        # Run MIDI processor
        with mido.open_input(input_port) as inport:
            tui.run_midi_processor(inport, output_ports)
            
    elif choice == 1:  # Audio Input
        # Get audio devices
        audio_devices = AudioToMidi.list_audio_devices()
        if not audio_devices:
            tui.show_error("No audio input devices available")
            return
            
        # Show device selection
        device_options = [f"{name} ({channels} channels)" for _, name, channels in audio_devices]
        device_choice = tui.show_menu("Select Audio Device", device_options)
        if device_choice < 0:
            return
            
        device_id = audio_devices[device_choice][0]
        device_info = sd.query_devices(device_id)
        
        # Ask if user wants to monitor levels
        monitor_choice = tui.show_menu("Would you like to monitor input levels?", ["Yes", "No"])
        if monitor_choice == 0:  # Yes
            tui.run_audio_monitor(
                device_id,
                device_info['max_input_channels'],
                int(device_info['default_samplerate'])
            )
            
        # Select channel
        num_channels = device_info['max_input_channels']
        channel = tui.show_value_input("Select channel number", min_val=0, max_val=num_channels-1)
        if channel is None:
            return
            
        # Select output ports
        num_ports = tui.show_value_input("Enter number of output ports", min_val=1, max_val=8)
        if num_ports is None:
            return
            
        output_ports = []
        available_ports = mido.get_output_names()
        for i in range(num_ports):
            port_choice = tui.show_menu(f"Select Output Port {i+1}", available_ports)
            if port_choice < 0:
                return
            output_ports.append(available_ports[port_choice])
            
    elif choice == 2:  # Generated Music
        # Select key
        key_choice = tui.show_menu("Select Key", NOTES)
        if key_choice < 0:
            return
        key = key_choice
        
        # Select tempo
        tempo = tui.show_value_input("Enter tempo (BPM)", min_val=10, max_val=300)
        if tempo is None:
            return
        
        # Number of outputs
        num_ports = tui.show_value_input("Enter number of output ports", min_val=2, max_val=8)
        if num_ports is None:
            return
            
        # Select output ports
        output_ports = []
        available_ports = mido.get_output_names()
        for i in range(num_ports):
            port_choice = tui.show_menu(f"Select Output Port {i+1}", available_ports)
            if port_choice < 0:
                return
            output_ports.append(available_ports[port_choice])
            
        # Select harmony modes
        harmony_modes = []
        harmony_options = [
            "Forward melody as-is",
            "Add diatonic thirds",
            "Add diatonic fourths",
            "Add random diatonic intervals",
            "Add random diatonic intervals (no seconds)",
            "Add contrary motion with random intervals",
            "Add strict counterpoint rules"
        ]
        
        for i in range(num_ports - 1):
            mode_choice = tui.show_menu(f"Select Harmony Mode for Voice {i+1}", harmony_options)
            if mode_choice < 0:
                return
            harmony_modes.append(mode_choice + 1)
            
        # Select chord progression
        progression_choice = tui.show_menu("Select Chord Progression", list(CHORD_PROGRESSIONS.keys()))
        if progression_choice < 0:
            return
        progression = list(CHORD_PROGRESSIONS.keys())[progression_choice]
        
        # Select rhythm pattern
        rhythm_choice = tui.show_menu("Select Rhythm Pattern", list(RHYTHM_PATTERNS.keys()))
        if rhythm_choice < 0:
            return
        rhythm_pattern = list(RHYTHM_PATTERNS.values())[rhythm_choice]
        
        # Start music generation
        command_queue = queue.Queue()
        stop_event = threading.Event()
        
        # Start keyboard input thread
        keyboard_thread = threading.Thread(
            target=keyboard_input_thread,
            args=(command_queue, stop_event),
            daemon=True
        )
        keyboard_thread.start()
        
        # Run music player interface with actual playback
        try:
            with contextlib.ExitStack() as stack:
                ports = [stack.enter_context(mido.open_output(port)) for port in output_ports]
                
                while True:
                    # Generate and play music
                    try:
                        melody = generate_melody(key, length=16, tempo=tempo, progression=progression, rhythm_pattern=rhythm_pattern)
                        harmonies = [
                            generate_harmony(melody, key, voice_number=i+1, harmony_mode=harmony_modes[i])
                            for i in range(len(output_ports)-1)
                        ]
                        
                        # Play all parts simultaneously
                        for step in range(len(melody)):
                            # Check for user input
                            if not command_queue.empty():
                                cmd = command_queue.get()
                                if cmd == 'q':
                                    return
                                elif cmd == 'n':
                                    break
                                elif cmd == 'k':
                                    key = (key + 1) % 12
                                    break
                            
                            # Get the note data for this step
                            m_note, duration, m_vel = melody[step]
                            
                            # Track all active notes for proper cleanup
                            active_notes = []
                            
                            # Send melody note to first port (if available)
                            if ports and len(ports) > 0:
                                try:
                                    msg = mido.Message('note_on', note=m_note, velocity=m_vel)
                                    ports[0].send(msg)
                                    active_notes.append((0, m_note))  # Track (port_index, note)
                                    
                                    # Update TUI display for melody note
                                    if m_note not in tui.active_notes:
                                        tui.active_notes[m_note] = []
                                    tui.active_notes[m_note].append('melody')
                                    
                                except Exception as e:
                                    tui.show_error(f"Error sending melody note: {str(e)}")
                                    continue
                            
                            # Send harmony notes to additional ports
                            for voice, harmony in enumerate(harmonies):
                                if voice + 1 < len(ports):  # Check if port exists
                                    try:
                                        h_note, _, h_vel = harmony[step]
                                        msg = mido.Message('note_on', note=h_note, velocity=h_vel)
                                        ports[voice + 1].send(msg)
                                        active_notes.append((voice + 1, h_note))
                                        
                                        # Update TUI display for harmony note
                                        if h_note not in tui.active_notes:
                                            tui.active_notes[h_note] = []
                                        tui.active_notes[h_note].append(f'harmony{voice+1}')
                                        
                                    except Exception as e:
                                        tui.show_error(f"Error sending harmony note to voice {voice + 1}: {str(e)}")
                                        continue
                            
                            # Wait for note duration
                            time.sleep(duration * 60 / tempo)
                            
                            # Send note-off messages for all active notes
                            for port_idx, note in active_notes:
                                try:
                                    if port_idx < len(ports):
                                        msg = mido.Message('note_off', note=note, velocity=0)
                                        ports[port_idx].send(msg)
                                        
                                        # Update TUI display
                                        if note in tui.active_notes:
                                            del tui.active_notes[note]
                                            
                                except Exception as e:
                                    tui.show_error(f"Error sending note-off to port {port_idx}: {str(e)}")
                                    continue
                            
                            # Update the TUI display
                            tui.update_screen()
                            
                    except Exception as e:
                        tui.show_error(f"Generation error: {str(e)}")
                        continue
        except Exception as e:
            tui.show_error(f"Port error: {str(e)}")
        finally:
            # Clean shutdown
            try:
                # Send all notes off on all channels for all ports
                if 'ports' in locals():
                    for port in ports:
                        try:
                            for channel in range(16):  # MIDI has 16 channels
                                for note in range(128):  # MIDI has 128 possible notes
                                    port.send(mido.Message('note_off', note=note, velocity=0, channel=channel))
                        except Exception:
                            continue  # Skip if port is already closed
            except Exception:
                pass  # Ignore cleanup errors
            
            stop_event.set()
            if 'keyboard_thread' in locals() and keyboard_thread.is_alive():
                keyboard_thread.join(timeout=1.0)
            
    else:  # Mode-based MIDI Processing
        # Select input port
        available_ports = mido.get_input_names()
        if not available_ports:
            tui.show_error("No MIDI input ports available")
            return
            
        port_choice = tui.show_menu("Select MIDI Input Port", available_ports)
        if port_choice < 0:
            return
            
        input_port = available_ports[port_choice]
        
        # Number of outputs
        num_ports = tui.show_value_input("Enter number of output ports", min_val=2, max_val=8)
        if num_ports is None:
            return
            
        # Select output ports
        output_ports = []
        available_ports = mido.get_output_names()
        for i in range(num_ports):
            port_choice = tui.show_menu(f"Select Output Port {i+1}", available_ports)
            if port_choice < 0:
                return
            output_ports.append(available_ports[port_choice])
            
        # Select initial key
        key_choice = tui.show_menu("Select Key", NOTES)
        if key_choice < 0:
            return
        key = key_choice
        
        # Select initial mode
        mode_choice = tui.show_menu("Select Initial Mode", [
            "Forward MIDI as-is",
            "Add diatonic thirds",
            "Add diatonic fourths",
            "Add random diatonic intervals",
            "Add random diatonic intervals (no seconds)",
            "Add contrary motion with random intervals",
            "Add strict counterpoint rules"
        ])
        if mode_choice < 0:
            return
        mode = mode_choice + 1
        
        # Start processing
        command_queue = queue.Queue()
        stop_event = threading.Event()
        
        # Start keyboard input thread
        keyboard_thread = threading.Thread(
            target=keyboard_input_thread,
            args=(command_queue, stop_event),
            daemon=True
        )
        keyboard_thread.start()
        
        # Run mode-based processor
        with mido.open_input(input_port) as inport:
            output_ports_list = [mido.open_output(port) for port in output_ports]
            try:
                # Initialize tracking variables
                prev_input_notes = {}  # Track previous input notes for each voice
                prev_output_notes = {}  # Track previous output notes for each voice
                
                while True:
                    # Process MIDI messages
                    msg = inport.poll()
                    if msg:
                        try:
                            # Always forward original message to first output
                            if len(output_ports_list) > 0:
                                output_ports_list[0].send(msg)
                            
                            if msg.type in ['note_on', 'note_off']:
                                # Update active notes in TUI
                                if msg.type == 'note_on' and msg.velocity > 0:
                                    # Generate harmony notes based on mode
                                    harmony_notes = []
                                    source_note = msg.note
                                    
                                    # Generate harmony for each additional output port
                                    for voice in range(len(output_ports_list) - 1):
                                        prev_in = prev_input_notes.get(voice + 1)
                                        prev_out = prev_output_notes.get(voice + 1)
                                        
                                        # Generate harmony based on mode
                                        if mode == 1:
                                            harmony_note = source_note  # Forward as-is
                                            harmony_msg = msg.copy()  # Forward exact message
                                        else:
                                            # Generate harmony for other modes
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
                                            harmony_msg = msg.copy(note=int(harmony_note))
                                        
                                        # Send harmony note
                                        if voice + 1 < len(output_ports_list):
                                            try:
                                                output_ports_list[voice + 1].send(harmony_msg)
                                                harmony_notes.append(harmony_note)
                                                
                                                # Update tracking for next voice
                                                if mode in [5, 6, 7]:  # Modes that need motion tracking
                                                    prev_input_notes[voice + 1] = source_note
                                                    prev_output_notes[voice + 1] = harmony_note
                                                
                                                # Next harmony will be generated from this one
                                                source_note = harmony_note
                                            except Exception as e:
                                                print(f"Error sending harmony note: {e}")
                                                continue
                                    
                                    # Store active notes for display
                                    tui.active_notes[msg.note] = harmony_notes
                                    
                                elif msg.type == 'note_off' or msg.velocity == 0:
                                    # Send note-off to all harmony notes
                                    if msg.note in tui.active_notes:
                                        harmony_notes = tui.active_notes[msg.note]
                                        for voice, harmony_note in enumerate(harmony_notes):
                                            if voice + 1 < len(output_ports_list):
                                                try:
                                                    harmony_msg = msg.copy(note=int(harmony_note))
                                                    output_ports_list[voice + 1].send(harmony_msg)
                                                except Exception as e:
                                                    print(f"Error sending note-off: {e}")
                                                    continue
                                        del tui.active_notes[msg.note]
                                        
                                        # Clear motion tracking for this note
                                        for voice in range(len(output_ports_list) - 1):
                                            if prev_input_notes.get(voice + 1) == msg.note:
                                                prev_input_notes[voice + 1] = None
                                                prev_output_notes[voice + 1] = None
                            else:
                                # Forward all other MIDI messages to all outputs
                                for port in output_ports_list[1:]:
                                    try:
                                        port.send(msg)
                                    except Exception as e:
                                        print(f"Error forwarding MIDI message: {e}")
                                        continue
                        except Exception as e:
                            print(f"Error processing MIDI message: {e}")
                            continue
                    
                    # Update UI and check for commands
                    cmd, value = tui.run_mode_based_processor(key, mode, command_queue)
                    if cmd == 'q':
                        break
                    elif cmd == 'k':
                        key = (key + 1) % 12
                    elif cmd == 'm':
                        mode = value
                        # Clear tracking when changing modes
                        prev_input_notes.clear()
                        prev_output_notes.clear()
                        tui.active_notes.clear()
            finally:
                # Send all notes off
                for port in output_ports_list:
                    try:
                        for channel in range(16):
                            for note in range(128):
                                port.send(mido.Message('note_off', note=note, velocity=0, channel=channel))
                    except:
                        continue
                # Clear state
                tui.active_notes.clear()
                for port in output_ports_list:
                    port.close()
                stop_event.set()

def main():
    """Entry point for the application."""
    try:
        curses.wrapper(curses_main)
    except KeyboardInterrupt:
        print("\nProgram terminated by user")
    except Exception as e:
        print(f"\nError: {e}")
    finally:
        # Restore terminal state
        try:
            termios.tcsetattr(sys.stdin.fileno(), termios.TCSADRAIN, termios.tcgetattr(sys.stdin.fileno()))
        except:
            pass

if __name__ == "__main__":
    main()
