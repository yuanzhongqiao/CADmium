<script>
	export let item
	import {
		new_realization_needed,
		step_being_edited,
		sketch_being_edited,
		looking_for,
		found,
		project_rust
	} from "shared/stores"
	sketch_being_edited.set(item.name)

	let mode = "Select"
	let sketch_modes = [{ name: "Select" }, { name: "Draw" }, { name: "Constrain" }]
	let plane_name = item.data.plane_name

	$: if (mode === "Select") {
		looking_for.set(["line", "circle"])
	} else {
		looking_for.set([])
	}

	const on_looking_for_plane = () => {
		console.log("looking for plane")
		looking_for.set(["plane"])
	}

	const on_no_longer_looking_for_plane = () => {
		console.log("no longer looking for plane")
		looking_for.set([])

		let message_obj = {
			UpdateSketchPlane: {
				workbench_id: 0,
				sketch_name: $sketch_being_edited,
				plane_name: plane_name
			}
		}

		console.log("sending message:", message_obj)
		$project_rust.send_message(JSON.stringify(message_obj))
	}

	found.subscribe((val) => {
		if (val && val.length > 0) {
			let new_plane = val[0]
			console.log("Found a plane: ", new_plane)
			item.data.plane_name = new_plane.name
			plane_name = new_plane.name
			looking_for.set([])
		}
	})
</script>

<div class="px-3 py-2 bg-gray-300 flex flex-col space-y-2">
	<div class="text-sm">Plane</div>
	<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
	<div
		tabindex="0"
		class="bg-gray-50 rounded flex shadow border focus:ring focus:border-blue-500"
		on:focus={on_looking_for_plane}
		on:blur={on_no_longer_looking_for_plane}
	>
		<div class="bg-sky-200 pl-2 py-0.5 m-1 rounded text-sm">
			{item.data.plane_name}
			<button class="rounded px-1 hover:bg-sky-400"><i class="fa-solid fa-x fa-xs" /></button>
		</div>
	</div>

	<div class="text-sm">Mode</div>
	<div class="flex">
		{#each sketch_modes as sketch_mode}
			{#if sketch_mode.name === mode}
				<div class="text-sm py-1 bg-white flex-grow border-solid border-2 rounded border-sky-500 text-center">
					{sketch_mode.name}
				</div>
			{:else}
				<!-- svelte-ignore a11y-no-static-element-interactions -->
				<!-- svelte-ignore a11y-click-events-have-key-events -->
				<div
					class="text-sm py-1 bg-gray-200 flex-grow rounded border-solid border-2 border-gray-500 text-center hover:bg-gray-300"
					on:click={() => {
						mode = sketch_mode.name
					}}
				>
					{sketch_mode.name}
				</div>
			{/if}
		{/each}
	</div>

	{#if mode === "Select"}
		<button
			class="bg-gray-200 hover:bg-gray-100 py-1 px-1 rounded shadow"
			on:click={() => {
				console.log("deleting")
				const message_objs = []
				console.log("item.data.line_segments", item.data.sketch.line_segments)
				for (const [segment_id, line_segment] of Object.entries(item.data.sketch.line_segments)) {
					console.log("a segment:", line_segment)
				}

				for (let i = 0; i < item.data.sketch.line_segments.length; i++) {
					let line_segment = item.data.sketch.line_segments[i]
					// console.log('a segment:', line_segment)
					if (line_segment.selected) {
						let message_obj = {
							DeleteLineSegment: {
								workbench_id: 0,
								sketch_name: $sketch_being_edited,
								line_segment_id: line_segment.id
							}
						}
						console.log("sending message:", message_obj)
						message_objs.push(message_obj)
						// $project_rust.send_message(JSON.stringify(message_obj))
					}
				}
			}}
			><div class="fa-solid fa-trash" />
			Delete</button
		>
	{/if}

	<button
		class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-1 px-1 rounded shadow"
		on:click={() => {
			step_being_edited.set(-1)
			new_realization_needed.set(true)
			sketch_being_edited.set(null)
		}}>Done</button
	>
</div>
