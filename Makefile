
shader.metallib: shader.air
	xcrun -sdk macosx metallib $^ -o $@

shader.air: shader.metal
	xcrun -sdk macosx metal -c $< -o $@

.PHONY: clean
clean:
	rm *.air *.metallib
