export class WebrtcBinding {
    constructor() {
        this._peer = new RTCPeerConnection();
        this._channel = this._peer.createDataChannel("channel", {
            ordered: false,
            maxRetransmits: 0,
        });
        this._channel.binaryType = "arraybuffer";
    }

    connect(url) {
        var self = this;
        let prom = this._peer.createOffer().then(function(offer) {
            return self._peer.setLocalDescription(offer);
        }).then(function() {
            var request = new XMLHttpRequest();
            request.open("POST", url, true);
            request.onload = function() {
                if (request.status == 200) {
                    var response = JSON.parse(request.responseText);
                    var sdp = new RTCSessionDescription(response.answer);
                    self._peer.setRemoteDescription(sdp).then(function() {
                        var candidate = new RTCIceCandidate(response.candidate);
                        self._peer.addIceCandidate(candidate).catch(function(err) {
                            console.log(err);
                        });
                    }).catch(function(err) {
                        console.log(err);
                    });
                }
            };
            request.send(self._peer.localDescription.sdp);
        }).catch(function(err) {
            console.log(err);
        });

        return prom;
    }

    close() {
        self._peer.close();
    }

    get channel() {
        return this._channel;
    }
    set channel(channel) {
        return this._channel = channel;
    }
}
