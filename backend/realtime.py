from diffusers import StableCascadeCombinedPipeline

pipe = StableCascadeCombinedPipeline.from_pretrained("stabilityai/stable-cascade", variant="bf16", torch_dtype=torch.bfloat16)

prompt = "an image of a shiba inu, donning a spacesuit and helmet"
pipe(
    prompt=prompt,
    negative_prompt="",
    num_inference_steps=10,
    prior_num_inference_steps=20,
    prior_guidance_scale=3.0,
    width=1024,
    height=1024,
).images[0].save("cascade-combined.png")
