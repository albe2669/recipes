import dagger
from dagger import dag, function, object_type


@object_type
class Recipes:
    chef_version: str = "v0.10.0"

    def create_container(self) -> dagger.Container:
        """Returns a container that echoes whatever string argument is provided"""

        chef_url = f"https://github.com/Zheoni/cooklang-chef/releases/download/{self.chef_version}/chef-x86_64-unknown-linux-musl.tar.gz"

        return (
            dag.container()
            .from_("alpine:latest")
            .with_exec(["apk", "add", "--update", "--no-cache", "curl"])
            .with_exec(["curl", "-sSL", chef_url, "-o", "/tmp/chef.tar.gz"])
            .with_exec(["tar", "-xzf", "/tmp/chef.tar.gz", "-C", "/usr/local/bin"])
            .with_exec(["rm", "/tmp/chef.tar.gz"])
        )

    @function
    async def get_markdown(
        self,
        source: dagger.Directory,
        recipe_path: str,
        container: dagger.Container | None = None,
    ) -> str:
        if container is None:
            container = self.create_container()

        """Returns lines that match a pattern in the files of the provided Directory"""
        return await (
            container.with_mounted_directory("/mnt", source)
            .with_workdir("/mnt")
            .with_exec(["chef", "recipe", recipe_path, "--format", "markdown"])
            .stdout()
        )

    @function
    async def read_recipe(self, source: dagger.Directory, recipe_path: str) -> str:
        container = self.create_container()

        """Returns lines that match a pattern in the files of the provided Directory"""
        return await (
            container.with_mounted_directory("/mnt", source)
            .with_workdir("/mnt")
            .with_exec(["chef", "recipe", recipe_path])
            .stdout()
        )
