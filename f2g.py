import bpy
import os

bpy.ops.import_scene.fbx(
    filepath=os.path.join(
        os.path.dirname(bpy.data.filepath),
        "assets/characters/Model/characterMedium.fbx",
    )
)

bpy.ops.import_scene.fbx(
    filepath=os.path.join(
        os.path.dirname(bpy.data.filepath),
        "assets/characters/Animations/idle.fbx",
    )
)

bpy.ops.import_scene.fbx(
    filepath=os.path.join(
        os.path.dirname(bpy.data.filepath),
        "assets/characters/Animations/jump.fbx",
    )
)

bpy.ops.import_scene.fbx(
    filepath=os.path.join(
        os.path.dirname(bpy.data.filepath),
        "assets/characters/Animations/run.fbx",
    )
)


bpy.ops.export_scene.gltf(
    filepath=os.path.join(
        os.path.dirname(bpy.data.filepath),
        "assets/characters/Model/characterMedium_out.gltf",
    )
)
