import sounddevice as sd
import numpy as np
import librosa
import mido
import queue
import threading

class AudioToMidi:
    def __init__(self, device_id=None, samplerate=48000, buffer_size=512, input_channel=0):
        self.samplerate = samplerate
        self.buffer_size = buffer_size
        self.device_id = device_id
        self.input_channel = input_channel
        self.audio_queue = queue.Queue()
        self.midi_queue = queue.Queue()
        self.running = False
        
        # Note tracking
        self.current_note = None
        self.note_on_threshold = 0.6  # Lower threshold for faster response
        self.silence_threshold = 0.05  # Lower silence threshold
        self.prev_magnitudes = None
        self.onset_threshold = 1.5  # Onset detection threshold
        
    def audio_callback(self, indata, frames, time, status):
        """This is called for each audio block."""
        if status:
            print(status)
        # Use the selected input channel
        audio_data = indata[:, self.input_channel]
        self.audio_queue.put(audio_data)
    
    def detect_pitch(self, audio_data):
        """Fast pitch detection using autocorrelation."""
        # Apply window function
        windowed = audio_data * np.hanning(len(audio_data))
        
        # Compute autocorrelation
        corr = np.correlate(windowed, windowed, mode='full')
        corr = corr[len(corr)//2:]
        
        # Find peaks in autocorrelation
        peaks = []
        peak_vals = []
        for i in range(1, len(corr)-1):
            if corr[i] > corr[i-1] and corr[i] > corr[i+1]:
                peaks.append(i)
                peak_vals.append(corr[i])
        
        if not peaks:
            return None, 0
        
        # Find highest peak in expected frequency range
        min_period = int(self.samplerate / 1000)  # ~1000 Hz
        max_period = int(self.samplerate / 50)    # ~50 Hz
        valid_peaks = [(i, v) for i, v in zip(peaks, peak_vals) 
                      if min_period <= i <= max_period]
        
        if not valid_peaks:
            return None, 0
            
        period = max(valid_peaks, key=lambda x: x[1])[0]
        frequency = self.samplerate / period
        confidence = max(peak_vals) / corr[0]
        
        return frequency, confidence
    
    def detect_onset(self, magnitudes):
        """Detect note onsets using spectral flux."""
        if self.prev_magnitudes is None:
            self.prev_magnitudes = magnitudes
            return False
            
        # Calculate spectral flux
        flux = np.sum(np.maximum(0, magnitudes - self.prev_magnitudes))
        self.prev_magnitudes = magnitudes
        
        return flux > self.onset_threshold
    
    def process_audio(self):
        """Process audio data and convert to MIDI notes."""
        while self.running:
            try:
                audio_data = self.audio_queue.get(timeout=1.0)
                
                # Calculate amplitude and spectrum
                amplitude = np.mean(np.abs(audio_data))
                spectrum = np.abs(np.fft.rfft(audio_data * np.hanning(len(audio_data))))
                
                if amplitude > self.silence_threshold:
                    # Detect pitch using autocorrelation
                    pitch, confidence = self.detect_pitch(audio_data)
                    
                    if pitch and confidence > self.note_on_threshold:
                        # Convert frequency to MIDI note number
                        midi_note = int(round(69 + 12 * np.log2(pitch/440.0)))
                        
                        # Check for onset
                        is_onset = self.detect_onset(spectrum)
                        
                        # Note onset detection with hysteresis
                        if is_onset or (self.current_note is None and confidence > self.note_on_threshold + 0.1):
                            # If there was a previous note, send note off
                            if self.current_note is not None and self.current_note != midi_note:
                                self.midi_queue.put(('note_off', self.current_note, 0))
                            
                            # Send note on for new note
                            if self.current_note != midi_note:
                                self.midi_queue.put(('note_on', midi_note, int(min(127, confidence * 100))))
                                self.current_note = midi_note
                
                # Send note off if amplitude drops below threshold
                elif self.current_note is not None:
                    self.midi_queue.put(('note_off', self.current_note, 0))
                    self.current_note = None
                
            except queue.Empty:
                continue
    
    def start(self):
        """Start audio input and processing."""
        self.running = True
        
        # Start audio input stream
        self.stream = sd.InputStream(
            device=self.device_id,
            channels=1,
            samplerate=self.samplerate,
            blocksize=self.buffer_size,
            callback=self.audio_callback
        )
        self.stream.start()
        
        # Start processing thread
        self.process_thread = threading.Thread(target=self.process_audio)
        self.process_thread.daemon = True
        self.process_thread.start()
    
    def stop(self):
        """Stop audio input and processing."""
        self.running = False
        if hasattr(self, 'stream'):
            self.stream.stop()
            self.stream.close()
        if hasattr(self, 'process_thread'):
            self.process_thread.join(timeout=1.0)
    
    def get_midi_message(self):
        """Get the next MIDI message if available."""
        try:
            msg_type, note, velocity = self.midi_queue.get_nowait()
            return mido.Message(msg_type, note=note, velocity=velocity)
        except queue.Empty:
            return None
    
    @staticmethod
    def list_audio_devices():
        """List all available audio input devices."""
        devices = sd.query_devices()
        input_devices = []
        for i, device in enumerate(devices):
            if device['max_input_channels'] > 0:
                input_devices.append((i, device['name'], device['max_input_channels']))
        return input_devices 