const hrEndpoint = "/api/v1/hr";

const hrPollIntervalMs = 500;

const hrStepIntervalSec = 0.05;

var audioContext = null;

function enableAudio() {
    audioContext = new AudioContext();
}

function makeSound() {
    if (!audioContext) {
	return;
    }

    try {
	const sound = document.getElementById("sound").cloneNode(true);
	const source = audioContext.createMediaElementSource(sound);
	source.connect(audioContext.destination);
	sound.cloneNode(true).play()
    } catch (error) {
	console.error(error);
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
    return fetch(hrEndpoint)
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
