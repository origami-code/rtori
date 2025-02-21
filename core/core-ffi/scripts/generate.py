import asyncio

class BuildDefinition:
    target: str
    profile: str


async def build_target(build: BuildDefinition):
    proc = await asyncio.create_subprocess_exec(
        "cargo",
        *["build", "--target", build.target, "--profile", build.profile],
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE
    )
    stdout, stderr = await proc.communicate()

    print(f'cargo exited with return code {proc.returncode}')
    if stderr:
        print(f'[stderr]\n{stderr.decode()}')
    

build = BuildDefinition()
build.target = "x86_64-pc-windows-msvc"
build.profile = "release"
asyncio.run(build_target(build))