# Taste

- When merging upstream changes, never use a merge strategy (no `-X ours`/`-X theirs`/`--strategy` overrides): let conflicts happen and resolve each one manually after reading the surrounding context. Confidence: 0.85
- When resolving merge conflicts, weigh local vs. remote per case and keep the better implementation ("取其精华，去其糟粕"); treat meaningful local changes/improvements as worth preserving rather than defaulting to upstream. Confidence: 0.85
- After a merge, the result must build cleanly — every library compiles and downstream call sites still work as the completion criterion for the task. Confidence: 0.8
- After merging, bump version numbers where local lags upstream. Confidence: 0.6
- Communicates in Chinese and expects replies in Chinese. Confidence: 0.8
