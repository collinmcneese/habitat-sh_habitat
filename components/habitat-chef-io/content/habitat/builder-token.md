+++
title = "Create an Access Token"
description = "Personal Access Token Management"

[menu]
  [menu.habitat]
    title = "Personal Access Token Management"
    identifier = "habitat/builder/builder-token"
    parent = "habitat/builder"
    weight = 20

+++

You can build and install Habitat artifacts without authenticating to Builder; however, some operations, like uploading your artifacts to Builder, or checking the status of your build jobs from the Habitat CLI, will require an access token.

From the Builder site, select your user icon in the upper right-hand side, and select **Profile**.

<img src="/images/screenshots/profile.png">

At the bottom of the profile page, select **Generate Token**.

<img src="/images/screenshots/generate-token.png">

Copy your token from this page.

## Windows

Save your Habitat authorization token as a permanent environment variable in Windows using:

```PS
SETX HAB_AUTH_TOKEN <token> /m
```

Replacing <token> with the contents of your generated token.

You can also save your Habitat authorization token as a permanent environment variable using the Windows user interface. In your Windows help bar, enter `environment` and select `Edit the system environment variables` from the list of suggestions.

This opens the `System Properties` window on the `Advanced` tab. Select the `Environment Variables` button.

<img src='/images/screenshots/environment_variable.png'>

In the next window, select the `New` button in the top part. This opens a dialog box that lets you set individual user variables.

<img src='/images/screenshots/environment_variable_new.png'>

Create a permanent environment variable by entering `HAB_AUTH_TOKEN` as the variable name. Next, paste the authorization token that you copied after you generated a new token on your Habitat profile page as the variable value. After you select the `OK`, you will see the new token in the user variables field.

<img src='/images/screenshots/environment_variable_new_var.png'>

To test that your new token works correctly, open the Command Prompt+++which you can find by entering command in the Windows search box+++and entering `echo %HAB_AUTH_TOKEN%`. You should see the value that you pasted into the environment variable.

<img src='/images/screenshots/environment_variable_set.png'>

## MacOS

Set the HAB_AUTH_TOKEN in the CLI with:

```bash
export HAB_AUTH_TOKEN=<token>
```

Replacing `<token>` with the contents of your generated token.

To use your token across sessions, set it as an environment variable in your interactive shell configuration file, such as your `.bashrc`.

```bash
export HAB_AUTH_TOKEN=<token>
```

Then initialize the path from the command line, by running:

```bash
source ~/.bashrc
```

+++
