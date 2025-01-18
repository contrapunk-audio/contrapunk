import tkinter as tk
from tkinter import ttk
import mido
import threading
import queue

class ContrapunkUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Contrapunk")
        self.root.geometry("600x800")
        
        # MIDI state
        self.input_port = None
        self.output_ports = []
        self.command_queue = queue.Queue()
        self.current_mode = tk.StringVar(value="1")
        self.key = tk.StringVar(value="0")
        
        # Create the main frame
        main_frame = ttk.Frame(root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Input port selection
        ttk.Label(main_frame, text="MIDI Input Port:").grid(row=0, column=0, sticky=tk.W, pady=5)
        self.input_ports_var = tk.StringVar()
        self.input_ports_combo = ttk.Combobox(main_frame, textvariable=self.input_ports_var)
        self.input_ports_combo['values'] = mido.get_input_names()
        self.input_ports_combo.grid(row=1, column=0, sticky=(tk.W, tk.E), pady=5)
        
        # Number of outputs
        ttk.Label(main_frame, text="Number of Outputs:").grid(row=2, column=0, sticky=tk.W, pady=5)
        self.num_outputs_var = tk.StringVar(value="2")
        num_outputs_spin = ttk.Spinbox(main_frame, from_=2, to=8, textvariable=self.num_outputs_var)
        num_outputs_spin.grid(row=3, column=0, sticky=(tk.W, tk.E), pady=5)
        
        # Output ports frame
        self.outputs_frame = ttk.LabelFrame(main_frame, text="Output Ports", padding="5")
        self.outputs_frame.grid(row=4, column=0, sticky=(tk.W, tk.E), pady=10)
        self.output_combos = []
        self.update_output_ports()
        
        # Key selection
        ttk.Label(main_frame, text="Key:").grid(row=5, column=0, sticky=tk.W, pady=5)
        key_frame = ttk.Frame(main_frame)
        key_frame.grid(row=6, column=0, sticky=(tk.W, tk.E), pady=5)
        
        keys = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
        for i, key_name in enumerate(keys):
            ttk.Radiobutton(key_frame, text=key_name, variable=self.key, value=str(i)).grid(row=i//6, column=i%6, padx=5)
        
        # Mode selection
        ttk.Label(main_frame, text="Mode:").grid(row=7, column=0, sticky=tk.W, pady=5)
        modes_frame = ttk.LabelFrame(main_frame, text="Modes", padding="5")
        modes_frame.grid(row=8, column=0, sticky=(tk.W, tk.E), pady=5)
        
        modes = [
            "1: Forward MIDI as-is",
            "2: Add diatonic thirds",
            "3: Add diatonic fourths",
            "4: Add random diatonic intervals",
            "5: Add random diatonic intervals (no seconds)",
            "6: Add contrary motion with random intervals",
            "7: Add strict counterpoint rules"
        ]
        
        for i, mode in enumerate(modes):
            ttk.Radiobutton(modes_frame, text=mode, variable=self.current_mode, 
                          value=str(i+1)).grid(row=i, column=0, sticky=tk.W, pady=2)
        
        # Status display
        self.status_var = tk.StringVar(value="Ready")
        status_label = ttk.Label(main_frame, textvariable=self.status_var)
        status_label.grid(row=9, column=0, sticky=(tk.W, tk.E), pady=10)
        
        # Control buttons
        buttons_frame = ttk.Frame(main_frame)
        buttons_frame.grid(row=10, column=0, sticky=(tk.W, tk.E), pady=10)
        
        ttk.Button(buttons_frame, text="Start", command=self.start_midi).grid(row=0, column=0, padx=5)
        ttk.Button(buttons_frame, text="Stop", command=self.stop_midi).grid(row=0, column=1, padx=5)
        
        # Bind num_outputs change
        self.num_outputs_var.trace('w', lambda *args: self.update_output_ports())
        
        self.running = False
    
    def update_output_ports(self):
        # Clear existing output combos
        for combo in self.output_combos:
            combo.destroy()
        self.output_combos.clear()
        
        # Get available output ports
        output_ports = mido.get_output_names()
        
        # Create new combos based on num_outputs
        try:
            num_outputs = max(2, min(8, int(self.num_outputs_var.get())))
        except ValueError:
            num_outputs = 2
        
        for i in range(num_outputs):
            label_text = "Melody Output:" if i == 0 else f"Harmony {i} Output:"
            ttk.Label(self.outputs_frame, text=label_text).grid(row=i*2, column=0, sticky=tk.W, pady=2)
            combo = ttk.Combobox(self.outputs_frame, values=output_ports)
            combo.grid(row=i*2+1, column=0, sticky=(tk.W, tk.E), pady=2)
            if output_ports:
                combo.set(output_ports[0])
            self.output_combos.append(combo)
    
    def start_midi(self):
        if self.running:
            return
        
        try:
            # Open input port
            input_port_name = self.input_ports_var.get()
            self.input_port = mido.open_input(input_port_name)
            
            # Open output ports
            self.output_ports = []
            for combo in self.output_combos:
                port_name = combo.get()
                port = mido.open_output(port_name)
                self.output_ports.append(port)
            
            # Start MIDI processing
            self.running = True
            self.midi_thread = threading.Thread(target=self.process_midi)
            self.midi_thread.daemon = True
            self.midi_thread.start()
            
            self.status_var.set("Running - Press number keys (1-7) to change modes")
            
            # Start key listener for mode changes
            self.root.bind('<Key>', self.handle_key)
            
        except Exception as e:
            self.status_var.set(f"Error: {str(e)}")
    
    def stop_midi(self):
        if not self.running:
            return
        
        self.running = False
        if self.input_port:
            self.input_port.close()
        for port in self.output_ports:
            port.close()
        
        self.root.unbind('<Key>')
        self.status_var.set("Stopped")
    
    def handle_key(self, event):
        if event.char in '1234567':
            self.current_mode.set(event.char)
            self.status_var.set(f"Mode changed to {event.char}")
    
    def process_midi(self):
        from main import (get_scale_notes, find_nearest_diatonic_third,
                        find_nearest_diatonic_fourth, find_random_diatonic_below,
                        find_random_diatonic_below_no_seconds, find_contrary_diatonic_below_no_seconds,
                        find_strict_counterpoint_below)
        
        key = int(self.key.get())
        scale_notes = get_scale_notes(key)
        active_notes = {}  # Maps input note to list of generated notes
        prev_input_notes = {}  # Track previous input notes for each voice
        prev_output_notes = {}  # Track previous output notes for each voice
        
        while self.running:
            msg = self.input_port.poll()
            if msg is None:
                continue
            
            if msg.type == 'note_on' or msg.type == 'note_off':
                # Always send original note to first output
                self.output_ports[0].send(msg)
                
                mode = int(self.current_mode.get())
                
                # Generate harmony notes based on mode
                if mode == 1:
                    # Forward as-is to all outputs
                    for port in self.output_ports[1:]:
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
                        for voice in range(len(self.output_ports) - 1):
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
                            self.output_ports[voice + 1].send(harmony_msg)
                            
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
                                self.output_ports[voice + 1].send(harmony_msg)
                            
                            if msg.type == 'note_off' or msg.velocity == 0:
                                del active_notes[msg.note]
                                if msg.note in prev_input_notes.values():
                                    # Clear motion tracking for all affected voices
                                    for voice in range(len(self.output_ports) - 1):
                                        if prev_input_notes.get(voice + 1) == msg.note:
                                            prev_input_notes[voice + 1] = None
                                            prev_output_notes[voice + 1] = None
            else:
                # Forward all other MIDI messages to all outputs
                for port in self.output_ports:
                    port.send(msg)

def run_ui():
    root = tk.Tk()
    app = ContrapunkUI(root)
    root.mainloop()

if __name__ == '__main__':
    run_ui() 