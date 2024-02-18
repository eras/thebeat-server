const hrEndpoint = "/api/v1/hr";

const hrPollIntervalMs = 500;

const hrStepIntervalSec = 0.05;

var audioContext = null;

const audioPoolSize = 5;
const heartbeatAudioPool = Array.from({ length: audioPoolSize }, () => null);
var heartbeatAudioPoolIndex = 0;

var soundBuffer = null;

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


async function enableAudio() {
    try {
	audioContext = new AudioContext();
    }
    catch(e) {
	alert('Web Audio API is not supported in this browser');
    }

    soundBuffer = await loadSoundBuffer("heart-beat.wav");
}

function makeSound() {
    if (!audioContext) {
	//console.error("No audio context");
	return;
    }
    if (!soundBuffer) {
	console.error("No sound buffer");
	audioContext = null;
	return;
    }

    try {
	var source = audioContext.createBufferSource();
	source.buffer = soundBuffer;
	source.connect(audioContext.destination);
	source.start();
    } catch (error) {
	console.error(error);
	audioContext = null;
    }
}

class HR {
    #hr;
    #time;

    constructor() {
	this.#time = 0;
	this.#hr = 0;
    }

    set hr(value) {
	this.#hr = value;
    }

    step() {
	this.#time += hrStepIntervalSec * this.#hr / 60;
	if (this.#time >= 1) {
	    makeSound();
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
    let visited = {};
    for (let [key, value] of Object.entries(response.data)) {
	visited[key] = true;
	if (!(key in hrs)) {
	    hrs[key] = new HR();
	}
	hrs[key].hr = value.hr;
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

function start() {
    retrievePeriodically(500);
    stepPeriodically();
}
