# Version 0.5.0 (2022-03-27)
* Update convert_stereo_to_mono_audio to return a Result
    * Used to panic when length of provided slice is not a multiple of two.

# Version 0.4.0 (2023-02-08)

# Version 0.3.0 (2022-12-14)

# Version 0.2.0 (2022-10-28)
* Update upstream whisper.cpp to 2c281d190b7ec351b8128ba386d110f100993973.
* Fix breaking changes in update, which cascade to users:
  * `DecodeStrategy` has been renamed to `SamplingStrategy`
  * `WhisperContext::sample_best`'s signature has changed: `needs_timestamp` has been removed.
* New features
  * `WhisperContext::full_n_tokens`
  * `WhisperContext::full_get_token_text`
  * `WhisperContext::full_get_token_id`
  * `WhisperContext::full_get_token_prob`
