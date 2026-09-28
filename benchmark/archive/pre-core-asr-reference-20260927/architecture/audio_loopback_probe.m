#import <CoreAudio/CoreAudio.h>
#import <AudioToolbox/AudioToolbox.h>
#import <Foundation/Foundation.h>
#include <math.h>
#include <pthread.h>
#include <stdatomic.h>

static NSString *stringProperty(AudioDeviceID device,
                                AudioObjectPropertySelector selector) {
    AudioObjectPropertyAddress address = {
        selector,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    };
    CFStringRef value = NULL;
    UInt32 size = sizeof(value);
    OSStatus status = AudioObjectGetPropertyData(device, &address, 0, NULL, &size,
                                                  &value);
    if (status != noErr || value == NULL) {
        return @"";
    }
    return CFBridgingRelease(value);
}

static UInt32 channelCount(AudioDeviceID device, AudioObjectPropertyScope scope) {
    AudioObjectPropertyAddress address = {
        kAudioDevicePropertyStreamConfiguration,
        scope,
        kAudioObjectPropertyElementMain,
    };
    UInt32 size = 0;
    if (AudioObjectGetPropertyDataSize(device, &address, 0, NULL, &size) != noErr ||
        size == 0) {
        return 0;
    }
    AudioBufferList *buffers = calloc(1, size);
    if (buffers == NULL) {
        return 0;
    }
    OSStatus status = AudioObjectGetPropertyData(device, &address, 0, NULL, &size,
                                                  buffers);
    UInt32 count = 0;
    if (status == noErr) {
        for (UInt32 index = 0; index < buffers->mNumberBuffers; index++) {
            count += buffers->mBuffers[index].mNumberChannels;
        }
    }
    free(buffers);
    return count;
}

static Float64 nominalSampleRate(AudioDeviceID device) {
    AudioObjectPropertyAddress address = {
        kAudioDevicePropertyNominalSampleRate,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    };
    Float64 value = 0;
    UInt32 size = sizeof(value);
    if (AudioObjectGetPropertyData(device, &address, 0, NULL, &size, &value) != noErr) {
        return 0;
    }
    return value;
}

static NSArray<NSDictionary *> *describeDevices(void) {
    AudioObjectPropertyAddress address = {
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    };
    UInt32 size = 0;
    OSStatus status = AudioObjectGetPropertyDataSize(kAudioObjectSystemObject,
                                                      &address, 0, NULL, &size);
    if (status != noErr) {
        return @[];
    }
    UInt32 count = size / sizeof(AudioDeviceID);
    AudioDeviceID *devices = calloc(count, sizeof(AudioDeviceID));
    if (devices == NULL) {
        return @[];
    }
    status = AudioObjectGetPropertyData(kAudioObjectSystemObject, &address, 0, NULL,
                                        &size, devices);
    NSMutableArray<NSDictionary *> *descriptions = [NSMutableArray array];
    if (status == noErr) {
        for (UInt32 index = 0; index < count; index++) {
            AudioDeviceID device = devices[index];
            [descriptions addObject:@{
                @"id": @(device),
                @"uid": stringProperty(device, kAudioDevicePropertyDeviceUID),
                @"name": stringProperty(device, kAudioObjectPropertyName),
                @"inputChannels": @(channelCount(device, kAudioDevicePropertyScopeInput)),
                @"outputChannels": @(channelCount(device, kAudioDevicePropertyScopeOutput)),
                @"nominalSampleRate": @(nominalSampleRate(device)),
            }];
        }
    }
    free(devices);
    return descriptions;
}

typedef struct {
    float *emitted;
    UInt32 emittedFrames;
    UInt32 emittedCursor;
    UInt32 outputStopFrame;
    float *captured;
    UInt32 capturedCapacityFrames;
    UInt32 capturedFrames;
    UInt32 channels;
    UInt64 captureStartHostTime;
    UInt64 outputStartHostTime;
    pthread_mutex_t mutex;
} DeviceIOState;

static UInt32 framesInBufferList(const AudioBufferList *buffers) {
    if (buffers == NULL || buffers->mNumberBuffers == 0) {
        return 0;
    }
    const AudioBuffer *buffer = &buffers->mBuffers[0];
    UInt32 channels = MAX(buffer->mNumberChannels, 1u);
    return buffer->mDataByteSize / (sizeof(float) * channels);
}

static float sampleFromBufferList(const AudioBufferList *buffers, UInt32 frame,
                                  UInt32 channel) {
    if (buffers->mNumberBuffers == 1) {
        const AudioBuffer *buffer = &buffers->mBuffers[0];
        UInt32 channels = MAX(buffer->mNumberChannels, 1u);
        return ((float *)buffer->mData)[frame * channels + MIN(channel, channels - 1)];
    }
    UInt32 index = MIN(channel, buffers->mNumberBuffers - 1);
    return ((float *)buffers->mBuffers[index].mData)[frame];
}

static void setSampleInBufferList(AudioBufferList *buffers, UInt32 frame,
                                  UInt32 channel, float value) {
    if (buffers->mNumberBuffers == 1) {
        AudioBuffer *buffer = &buffers->mBuffers[0];
        UInt32 channels = MAX(buffer->mNumberChannels, 1u);
        ((float *)buffer->mData)[frame * channels + MIN(channel, channels - 1)] = value;
        return;
    }
    UInt32 index = MIN(channel, buffers->mNumberBuffers - 1);
    ((float *)buffers->mBuffers[index].mData)[frame] = value;
}

static OSStatus deviceIOCallback(AudioDeviceID device,
                                 const AudioTimeStamp *now,
                                 const AudioBufferList *inputData,
                                 const AudioTimeStamp *inputTime,
                                 AudioBufferList *outputData,
                                 const AudioTimeStamp *outputTime,
                                 void *userData) {
    (void)device;
    (void)now;
    (void)inputTime;
    (void)outputTime;
    DeviceIOState *state = userData;
    pthread_mutex_lock(&state->mutex);
    if (state->outputStartHostTime == 0 && outputTime != NULL &&
        (outputTime->mFlags & kAudioTimeStampHostTimeValid) != 0) {
        state->outputStartHostTime = outputTime->mHostTime;
    }
    pthread_mutex_unlock(&state->mutex);
    UInt32 inputFrames = framesInBufferList(inputData);
    pthread_mutex_lock(&state->mutex);
    if (state->capturedFrames == 0 && inputTime != NULL &&
        (inputTime->mFlags & kAudioTimeStampHostTimeValid) != 0) {
        state->captureStartHostTime = inputTime->mHostTime;
    }
    UInt32 available = state->capturedCapacityFrames - state->capturedFrames;
    UInt32 copiedFrames = MIN(inputFrames, available);
    for (UInt32 frame = 0; frame < copiedFrames; frame++) {
        for (UInt32 channel = 0; channel < state->channels; channel++) {
            state->captured[(state->capturedFrames + frame) * state->channels + channel] =
                sampleFromBufferList(inputData, frame, channel);
        }
    }
    state->capturedFrames += copiedFrames;
    pthread_mutex_unlock(&state->mutex);

    UInt32 outputFrames = framesInBufferList(outputData);
    for (UInt32 frame = 0; frame < outputFrames; frame++) {
        for (UInt32 channel = 0; channel < state->channels; channel++) {
            float value = state->emittedCursor < state->emittedFrames &&
                                  state->emittedCursor < state->outputStopFrame
                ? state->emitted[state->emittedCursor * state->channels + channel]
                : 0.0f;
            setSampleInBufferList(outputData, frame, channel, value);
        }
        state->emittedCursor++;
    }
    return noErr;
}

static AudioStreamBasicDescription floatStereoFormat(Float64 sampleRate) {
    AudioStreamBasicDescription format = {0};
    format.mSampleRate = sampleRate;
    format.mFormatID = kAudioFormatLinearPCM;
    format.mFormatFlags = kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked;
    format.mBytesPerPacket = 2 * sizeof(float);
    format.mFramesPerPacket = 1;
    format.mBytesPerFrame = 2 * sizeof(float);
    format.mChannelsPerFrame = 2;
    format.mBitsPerChannel = 32;
    return format;
}

static float *readAudioFile(NSString *path, UInt32 sampleRate, UInt32 channels,
                            UInt32 *frameCount, NSString **failure) {
    ExtAudioFileRef file = NULL;
    NSURL *url = [NSURL fileURLWithPath:path];
    OSStatus status = ExtAudioFileOpenURL((__bridge CFURLRef)url, &file);
    if (status != noErr) {
        *failure = [NSString stringWithFormat:@"ExtAudioFileOpenURL failed: %d", status];
        return NULL;
    }
    AudioStreamBasicDescription sourceFormat = {0};
    UInt32 propertySize = sizeof(sourceFormat);
    status = ExtAudioFileGetProperty(file, kExtAudioFileProperty_FileDataFormat,
                                     &propertySize, &sourceFormat);
    SInt64 sourceFrames = 0;
    propertySize = sizeof(sourceFrames);
    if (status == noErr) {
        status = ExtAudioFileGetProperty(file, kExtAudioFileProperty_FileLengthFrames,
                                         &propertySize, &sourceFrames);
    }
    AudioStreamBasicDescription clientFormat = floatStereoFormat(sampleRate);
    if (status == noErr) {
        status = ExtAudioFileSetProperty(file, kExtAudioFileProperty_ClientDataFormat,
                                         sizeof(clientFormat), &clientFormat);
    }
    if (status != noErr || sourceFormat.mSampleRate <= 0 || sourceFrames <= 0) {
        *failure = [NSString stringWithFormat:@"read input audio metadata failed: %d",
                                              status];
        ExtAudioFileDispose(file);
        return NULL;
    }
    UInt32 capacity = (UInt32)ceil(sourceFrames * sampleRate / sourceFormat.mSampleRate) +
                      4096;
    float *samples = calloc(capacity * channels, sizeof(float));
    if (samples == NULL) {
        *failure = @"input audio allocation failed";
        ExtAudioFileDispose(file);
        return NULL;
    }
    AudioBufferList buffers = {0};
    buffers.mNumberBuffers = 1;
    buffers.mBuffers[0].mNumberChannels = channels;
    buffers.mBuffers[0].mDataByteSize = capacity * channels * sizeof(float);
    buffers.mBuffers[0].mData = samples;
    UInt32 frames = capacity;
    status = ExtAudioFileRead(file, &frames, &buffers);
    ExtAudioFileDispose(file);
    if (status != noErr || frames == 0) {
        *failure = [NSString stringWithFormat:@"ExtAudioFileRead failed: %d", status];
        free(samples);
        return NULL;
    }
    *frameCount = frames;
    return samples;
}

static BOOL writeFloatWav(NSString *path, const float *samples, UInt32 frames,
                          UInt32 channels, UInt32 sampleRate, NSError **error) {
    NSMutableData *data = [NSMutableData data];
    UInt32 audioBytes = frames * channels * sizeof(float);
    UInt32 riffSize = 36 + audioBytes;
    UInt16 formatTag = 3;
    UInt16 channelCount = channels;
    UInt32 byteRate = sampleRate * channels * sizeof(float);
    UInt16 blockAlign = channels * sizeof(float);
    UInt16 bitsPerSample = 32;
    UInt32 fmtSize = 16;
    [data appendBytes:"RIFF" length:4];
    [data appendBytes:&riffSize length:4];
    [data appendBytes:"WAVEfmt " length:8];
    [data appendBytes:&fmtSize length:4];
    [data appendBytes:&formatTag length:2];
    [data appendBytes:&channelCount length:2];
    [data appendBytes:&sampleRate length:4];
    [data appendBytes:&byteRate length:4];
    [data appendBytes:&blockAlign length:2];
    [data appendBytes:&bitsPerSample length:2];
    [data appendBytes:"data" length:4];
    [data appendBytes:&audioBytes length:4];
    [data appendBytes:samples length:audioBytes];
    return [data writeToFile:path options:NSDataWritingAtomic error:error];
}

static AudioDeviceID findDeviceByUID(NSString *wantedUID) {
    for (NSDictionary *device in describeDevices()) {
        if ([device[@"uid"] isEqualToString:wantedUID]) {
            return [device[@"id"] unsignedIntValue];
        }
    }
    return kAudioObjectUnknown;
}

static NSDictionary *qualifyLoopback(NSString *uid, NSString *outputDirectory,
                                      NSString *inputWav, BOOL interrupt,
                                      int *exitCode) {
    fprintf(stderr, "audio-loopback: prepare\n");
    const UInt32 sampleRate = 48000;
    const UInt32 channels = 2;
    UInt32 outputFrames = sampleRate;
    UInt32 markerStart = sampleRate / 4;
    UInt32 markerFrames = sampleRate / 4;
    const float amplitude = 0.2f;
    NSString *failure = nil;
    AudioDeviceID device = findDeviceByUID(uid);
    if (device == kAudioObjectUnknown) {
        failure = [NSString stringWithFormat:@"device not found: %@", uid];
    } else if (channelCount(device, kAudioDevicePropertyScopeInput) < channels ||
               channelCount(device, kAudioDevicePropertyScopeOutput) < channels) {
        failure = @"selected device is not 2-channel duplex";
    } else if (llround(nominalSampleRate(device)) != sampleRate) {
        failure = @"selected device nominal sample rate is not 48000 Hz";
    }

    float *emitted = NULL;
    if (failure == nil && inputWav != nil) {
        emitted = readAudioFile(inputWav, sampleRate, channels, &outputFrames, &failure);
        if (emitted != NULL) {
            markerStart = UINT32_MAX;
            UInt32 markerEnd = 0;
            for (UInt32 frame = 0; frame < outputFrames; frame++) {
                if (fabsf(emitted[frame * channels]) >= 0.001f) {
                    markerStart = MIN(markerStart, frame);
                    markerEnd = frame;
                }
            }
            if (markerStart == UINT32_MAX || markerEnd <= markerStart) {
                failure = @"input WAV has no meaningful non-silent waveform";
            } else {
                markerFrames = markerEnd - markerStart + 1;
            }
        }
    } else if (failure == nil) {
        emitted = calloc(outputFrames * channels, sizeof(float));
        if (emitted == NULL) {
            failure = @"audio sample allocation failed";
        }
    }
    if (failure == nil && inputWav == nil) {
        for (UInt32 frame = 0; frame < markerFrames; frame++) {
            double fraction = (double)frame / markerFrames;
            double frequency = 440.0 + 1320.0 * fraction;
            double phase = 2.0 * M_PI * frequency * frame / sampleRate;
            float window = sin(M_PI * fraction);
            float value = amplitude * window * sin(phase);
            emitted[(markerStart + frame) * channels] = value;
            emitted[(markerStart + frame) * channels + 1] = value;
        }
    }
    UInt32 captureCapacityFrames = MAX(sampleRate * 2, outputFrames + sampleRate);
    float *captured = calloc(captureCapacityFrames * channels, sizeof(float));
    if (captured == NULL) {
        failure = @"capture sample allocation failed";
    }

    DeviceIOState ioState = {0};
    ioState.emitted = emitted;
    ioState.emittedFrames = outputFrames;
    ioState.outputStopFrame = outputFrames;
    UInt32 bargeInFrame = UINT32_MAX;
    if (interrupt && failure == nil) {
        // Inject barge-in while the fixture is actually audible. Speech fixtures
        // can contain a pause at an arbitrary fixed offset, which would make the
        // sample invalid even though the device loopback itself is healthy.
        UInt32 searchStart = MIN(markerStart + sampleRate / 10, outputFrames - 1);
        UInt32 searchEnd = outputFrames > 513 ? outputFrames - 513 : outputFrames - 1;
        bargeInFrame = searchStart;
        for (UInt32 frame = searchStart; frame <= searchEnd; frame++) {
            if (fabsf(emitted[frame * channels]) >= 0.02f) {
                bargeInFrame = frame;
                break;
            }
        }
        ioState.outputStopFrame = MIN(bargeInFrame + 512, outputFrames);
        markerFrames = ioState.outputStopFrame - markerStart;
    }
    ioState.captured = captured;
    ioState.capturedCapacityFrames = captureCapacityFrames;
    ioState.channels = channels;
    pthread_mutex_init(&ioState.mutex, NULL);
    AudioDeviceIOProcID ioProc = NULL;
    if (failure == nil) {
        fprintf(stderr, "audio-loopback: create device IOProc\n");
        OSStatus status = AudioDeviceCreateIOProcID(device, deviceIOCallback,
                                                    &ioState, &ioProc);
        if (status != noErr) {
            failure = [NSString stringWithFormat:@"AudioDeviceCreateIOProcID failed: %d",
                                                  status];
        }
    }
    if (failure == nil) {
        fprintf(stderr, "audio-loopback: start device\n");
        OSStatus status = AudioDeviceStart(device, ioProc);
        if (status != noErr) {
            failure = [NSString stringWithFormat:@"AudioDeviceStart failed: %d",
                                                  status];
        }
    }
    if (failure == nil) {
        fprintf(stderr, "audio-loopback: device running\n");
        useconds_t durationUs = MAX(
            1500000u,
            (useconds_t)(1000000.0 * outputFrames / sampleRate) + 500000u);
        usleep(durationUs);
        fprintf(stderr, "audio-loopback: stop device\n");
        OSStatus status = AudioDeviceStop(device, ioProc);
        if (status != noErr) {
            failure = [NSString stringWithFormat:@"AudioDeviceStop failed: %d", status];
        }
    }

    pthread_mutex_lock(&ioState.mutex);
    UInt32 capturedFrames = ioState.capturedFrames;
    UInt64 captureStartHostTime = ioState.captureStartHostTime;
    UInt64 outputStartHostTime = ioState.outputStartHostTime;
    pthread_mutex_unlock(&ioState.mutex);

    double bestCorrelation = 0;
    UInt32 bestOffset = 0;
    float peak = 0;
    UInt32 onsetFrame = UINT32_MAX;
    UInt32 lastFrame = 0;
    if (failure == nil && capturedFrames >= markerFrames) {
        for (UInt32 frame = 0; frame < capturedFrames; frame++) {
            float value = fabsf(captured[frame * channels]);
            peak = MAX(peak, value);
            if (value >= 0.01f) {
                if (onsetFrame == UINT32_MAX) {
                    onsetFrame = frame;
                }
                lastFrame = frame;
            }
        }
        for (UInt32 offset = 0; offset + markerFrames <= capturedFrames; offset++) {
            double dot = 0;
            double emittedEnergy = 0;
            double capturedEnergy = 0;
            for (UInt32 frame = 0; frame < markerFrames; frame++) {
                float expected = emitted[(markerStart + frame) * channels];
                float observed = captured[(offset + frame) * channels];
                dot += expected * observed;
                emittedEnergy += expected * expected;
                capturedEnergy += observed * observed;
            }
            if (capturedEnergy > 0) {
                double correlation = dot / sqrt(emittedEnergy * capturedEnergy);
                if (correlation > bestCorrelation) {
                    bestCorrelation = correlation;
                    bestOffset = offset;
                }
            }
        }
    }

    NSFileManager *manager = NSFileManager.defaultManager;
    NSError *fileError = nil;
    [manager createDirectoryAtPath:outputDirectory
       withIntermediateDirectories:YES
                        attributes:nil
                             error:&fileError];
    if (failure == nil && fileError != nil) {
        failure = fileError.localizedDescription;
    }
    NSString *emittedPath = [outputDirectory stringByAppendingPathComponent:@"emitted.wav"];
    NSString *capturedPath = [outputDirectory stringByAppendingPathComponent:@"captured.wav"];
    if (failure == nil &&
        (!writeFloatWav(emittedPath, emitted, outputFrames, channels, sampleRate,
                        &fileError) ||
         !writeFloatWav(capturedPath, captured, capturedFrames, channels, sampleRate,
                        &fileError))) {
        failure = fileError.localizedDescription;
    }

    int64_t detectedLastNs =
        (onsetFrame == UINT32_MAX || captureStartHostTime == 0)
            ? -1
            : (int64_t)AudioConvertHostTimeToNanos(captureStartHostTime) +
                  (int64_t)(1000000000.0 * lastFrame / sampleRate);
    int64_t bargeInNs =
        (!interrupt || outputStartHostTime == 0)
            ? -1
            : (int64_t)AudioConvertHostTimeToNanos(outputStartHostTime) +
                  (int64_t)(1000000000.0 * bargeInFrame / sampleRate);
    int64_t interruptionNs =
        (detectedLastNs < 0 || bargeInNs < 0) ? -1 : detectedLastNs - bargeInNs;
    BOOL passed = failure == nil && capturedFrames >= sampleRate &&
                  onsetFrame != UINT32_MAX && peak >= 0.1f &&
                  bestCorrelation >= 0.95 && (!interrupt || interruptionNs >= 0);
    NSMutableDictionary *report = [@{
        @"schemaVersion": @"via.audio-loopback-qualification.v1",
        @"waveformType": inputWav == nil ? @"generated_chirp" : @"speech_fixture",
        @"interruptionMode": @(interrupt),
        @"inputWav": inputWav == nil ? [NSNull null] : inputWav,
        @"deviceUID": uid,
        @"deviceName": device == kAudioObjectUnknown
            ? @""
            : stringProperty(device, kAudioObjectPropertyName),
        @"sampleRateHz": @(sampleRate),
        @"channels": @(channels),
        @"emittedFrames": @(outputFrames),
        @"capturedFrames": @(capturedFrames),
        @"markerStartFrame": @(markerStart),
        @"markerFrames": @(markerFrames),
        @"detectedOnsetFrame": onsetFrame == UINT32_MAX ? [NSNull null] : @(onsetFrame),
        @"detectedLastFrame": onsetFrame == UINT32_MAX ? [NSNull null] : @(lastFrame),
        @"detectedOnsetMonotonicNs":
            (onsetFrame == UINT32_MAX || captureStartHostTime == 0)
                ? [NSNull null]
                : @(AudioConvertHostTimeToNanos(captureStartHostTime) +
                    (UInt64)(1000000000.0 * onsetFrame / sampleRate)),
        @"detectedLastMonotonicNs":
            (onsetFrame == UINT32_MAX || captureStartHostTime == 0)
                ? [NSNull null]
                : @(AudioConvertHostTimeToNanos(captureStartHostTime) +
                    (UInt64)(1000000000.0 * lastFrame / sampleRate)),
        @"bargeInMonotonicNs":
            bargeInNs < 0 ? [NSNull null] : @(bargeInNs),
        @"interruptionMs":
            interruptionNs < 0 ? [NSNull null] : @((double)interruptionNs / 1000000.0),
        @"peakAmplitude": @(peak),
        @"bestCorrelation": @(bestCorrelation),
        @"bestMarkerOffsetFrame": @(bestOffset),
        @"outputToCaptureOffsetMs": @(1000.0 *
            ((int64_t)bestOffset - (int64_t)markerStart) / sampleRate),
        @"passCriteria": @{
            @"minimumCapturedFrames": @(sampleRate),
            @"minimumPeakAmplitude": @0.1,
            @"minimumCorrelation": @0.95,
        },
        @"passed": @(passed),
        @"evidenceLabel": @"MEASURED_REFERENCE_HARNESS",
        @"artifacts": @{
            @"emittedWav": @"emitted.wav",
            @"capturedWav": @"captured.wav",
        },
    } mutableCopy];
    if (failure != nil) {
        report[@"failure"] = failure;
    }
    NSData *reportData = [NSJSONSerialization dataWithJSONObject:report
                                                          options:NSJSONWritingPrettyPrinted |
                                                                  NSJSONWritingSortedKeys
                                                            error:&fileError];
    if (reportData != nil) {
        [reportData writeToFile:[outputDirectory stringByAppendingPathComponent:@"report.json"]
                        options:NSDataWritingAtomic
                          error:nil];
    }

    if (ioProc != NULL) {
        fprintf(stderr, "audio-loopback: destroy IOProc\n");
        AudioDeviceDestroyIOProcID(device, ioProc);
    }
    fprintf(stderr, "audio-loopback: finished\n");
    pthread_mutex_destroy(&ioState.mutex);
    free(emitted);
    free(captured);
    *exitCode = passed ? 0 : 1;
    return report;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        NSObject *result = nil;
        int exitCode = 0;
        if (argc == 2 && strcmp(argv[1], "--list") == 0) {
            result = describeDevices();
        } else if (argc == 6 && strcmp(argv[1], "--qualify") == 0 &&
                   strcmp(argv[2], "--device-uid") == 0 &&
                   strcmp(argv[4], "--out-dir") == 0) {
            result = qualifyLoopback([NSString stringWithUTF8String:argv[3]],
                                     [NSString stringWithUTF8String:argv[5]],
                                     nil, NO, &exitCode);
        } else if (argc == 8 && strcmp(argv[1], "--play-capture") == 0 &&
                   strcmp(argv[2], "--wav") == 0 &&
                   strcmp(argv[4], "--device-uid") == 0 &&
                   strcmp(argv[6], "--out-dir") == 0) {
            result = qualifyLoopback([NSString stringWithUTF8String:argv[5]],
                                     [NSString stringWithUTF8String:argv[7]],
                                     [NSString stringWithUTF8String:argv[3]],
                                     NO, &exitCode);
        } else if (argc == 8 && strcmp(argv[1], "--interrupt-capture") == 0 &&
                   strcmp(argv[2], "--wav") == 0 &&
                   strcmp(argv[4], "--device-uid") == 0 &&
                   strcmp(argv[6], "--out-dir") == 0) {
            result = qualifyLoopback([NSString stringWithUTF8String:argv[5]],
                                     [NSString stringWithUTF8String:argv[7]],
                                     [NSString stringWithUTF8String:argv[3]],
                                     YES, &exitCode);
        } else {
            fprintf(stderr,
                    "usage: audio-loopback-probe --list | --qualify "
                    "--device-uid UID --out-dir DIRECTORY | --play-capture "
                    "--wav FILE --device-uid UID --out-dir DIRECTORY | "
                    "--interrupt-capture --wav FILE --device-uid UID "
                    "--out-dir DIRECTORY\n");
            return 2;
        }
        NSError *error = nil;
        NSData *json = [NSJSONSerialization dataWithJSONObject:result
                                                        options:NSJSONWritingPrettyPrinted |
                                                                NSJSONWritingSortedKeys
                                                          error:&error];
        if (json == nil) {
            fprintf(stderr, "%s\n", error.localizedDescription.UTF8String);
            return 1;
        }
        fwrite(json.bytes, 1, json.length, stdout);
        fputc('\n', stdout);
        return exitCode;
    }
}
