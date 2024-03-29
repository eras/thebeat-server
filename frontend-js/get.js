const volumeEndpoint = "/api/v1/volume";
const hrEndpoint = "/api/v1/hr";

const hrPollIntervalMs = 500;

const hrStepIntervalSec = 0.05;

var audioContext = null;

const audioPoolSize = 5;
const heartbeatAudioPool = Array.from({ length: audioPoolSize }, () => null);
var heartbeatAudioPoolIndex = 0;
var volumeSlider = null;
var volumeChangeIndex = -1;

function generateUUID() {
    let d = new Date().getTime();
    let uuid = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
	let r = (d + Math.random()*16)%16 | 0;
	d = Math.floor(d/16);
	return (c=='x' ? r : (r&0x3|0x8)).toString(16);
    });
    return uuid;
}
var webUuid = generateUUID();

var soundBuffers = {};

var missedResponseCount = 0;

function loadSoundBuffer(url) {
    return new Promise((resolve, reject) => {
	let request = new XMLHttpRequest();
	request.open('GET', url, true);
	request.responseType = 'arraybuffer';

	request.onload = function() {
	    audioContext.decodeAudioData(request.response, (buffer) => {
		resolve(buffer);
	    }, reject);
	}
	request.send();
    });
}

function getRoomName() {
    return document.getElementById("room-name").value;
}

var startedPolling = false;

function startPolling() {
    if (startedPolling) {
	return;
    }
    startedPolling = true;
    retrievePeriodically(500);
    stepPeriodically();
}

function stopPolling() {
    if (!startedPolling) {
	return;
    }
    startedPolling = false;
    retrievePeriodically(500);
    stepPeriodically();
}


async function enableAudio() {
    startPolling();
    try {
	audioContext = new AudioContext();
    }
    catch(e) {
	alert('Web Audio API is not supported in this browser');
    }

    const audioFiles = [
	"heart-beat.wav",
	"beep.wav",
	"heart-beat500.wav",
	"heart-beat1000.wav",
	"heart-beat1500.wav",
	"beep-100.wav",
	"beep-200.wav",
	"beep-300.wav",
	"beep-400.wav",
    ];
    for (audioFile of audioFiles) {
	soundBuffers[audioFile] = await loadSoundBuffer(audioFile);
    }
}

function makeSound(audio_file) {
    if (!audioContext) {
	console.error("No audio context");
	return;
    }
    if (Object.keys(soundBuffers).length === 0) {
	console.error("No sound buffers");
	audioContext = null;
	return;
    }

    try {
	var source = audioContext.createBufferSource();
	source.buffer = soundBuffers[audio_file];

	var dbVolume = parseFloat(volumeSlider.value);
	if (audio_file == "beep.wav") {
	    dbVolume -= 10; // hack
	}
	var volume = Math.pow(10, dbVolume / 20);
	const gain = audioContext.createGain();
	gain.gain.value = volume; // Default volume value
	
	source.connect(gain);
	gain.connect(audioContext.destination);
	source.start();
    } catch (error) {
	console.error(error);
	audioContext = null;
    }
}

class HR {
    #hr;
    #time;
    #audio_file;

    constructor() {
	this.#time = 0;
	this.#hr = 0;
	this.#audio_file = "heart-beat.wav";
    }

    set hr(value) {
	this.#hr = value;
    }

    set audio_file(value) {
	this.#audio_file = value;
    }

    step() {
	this.#time += hrStepIntervalSec * this.#hr / 60;
	if (this.#time >= 1) {
	    makeSound(this.#audio_file);
	    this.#time -= Math.trunc(this.#time);
	}
    }
}

var hrs = {};

// Function to retrieve the HTTP URL
function retrieveHR() {
    return fetch(`${hrEndpoint}/${getRoomName()}`)
	.then(response => {
	    if (!response.ok) {
		throw new Error('Network response was not ok');
	    }
	    return response.json();
	})
	.catch(error => {
	    console.error('There was a problem with the fetch operation:', error);
	});
}

function processResponse(response) {
    if (response === undefined) {
	missedResponseCount += 1
	if (missedResponseCount == 10) {
	    hrs = {};
	}
	return;
    }
    missedResponseCount = 0
    let visited = {};
    for (let [key, value] of Object.entries(response.data)) {
	visited[key] = true;
	if (!(key in hrs)) {
	    hrs[key] = new HR();
	}
	hrs[key].hr = value.hr;
	hrs[key].audio_file = value.audio_file;
    }
    if (response.volume_change_index != volumeChangeIndex) {
	volumeChangeIndex = response.volume_change_index;
	// Assume we're getting old stuff and our local value always beats our old value
	if (response.volume_changer_uuid != webUuid) {
	    volumeSlider.value = response.volume;
	    updateVolumeLabel();
	}
    }
    let allKeys = Object.keys(hrs);
    for (let key of allKeys) {
	if (!(key in visited)) {
	    delete hrs[key];
	}
    }
}

// Function to retrieve the URL twice a second
function retrievePeriodically() {
    setInterval(() => {
	retrieveHR().then(processResponse);
    }, hrPollIntervalMs);
}

function step() {
    for (let [key, value] of Object.entries(hrs)) {
	hrs[key].step();
    }
}

function stepPeriodically() {
    setInterval(() => {
	step()
    }, hrStepIntervalSec * 1000);
}

// Function to handle volume change
function updateVolumeLabel() {
    document.getElementById("volumeDb").innerHTML = `${volumeSlider.value}dB`;
}

// Function to handle volume change
function handleVolumeChange() {
    fetch(`${volumeEndpoint}/${getRoomName()}`, {
	method: 'POST',
	headers: {
	    'Content-Type': 'application/json'
	},
	body: JSON.stringify({"volume": parseFloat(volumeSlider.value),
			      "volume_changer_uuid": webUuid })
    })
    updateVolumeLabel();
}

function start() {
    // Get reference to the volume slider
    volumeSlider = document.getElementById('volumeSlider');

    // Attach event listener to volume slider
    volumeSlider.addEventListener('input', handleVolumeChange);
}
