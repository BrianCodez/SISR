# <svg xmlns="http://www.w3.org/2000/svg" width="1.4em" height="1.4em" style="transform: translate(0px, 10px)" viewBox="0 0 16 16"><g fill="currentColor"><path d="M.329 10.333A8.01 8.01 0 0 0 7.99 16C12.414 16 16 12.418 16 8s-3.586-8-8.009-8A8.006 8.006 0 0 0 0 7.468l.003.006l4.304 1.769A2.2 2.2 0 0 1 5.62 8.88l1.96-2.844l-.001-.04a3.046 3.046 0 0 1 3.042-3.043a3.046 3.046 0 0 1 3.042 3.043a3.047 3.047 0 0 1-3.111 3.044l-2.804 2a2.223 2.223 0 0 1-3.075 2.11a2.22 2.22 0 0 1-1.312-1.568L.33 10.333Z"/><path d="M4.868 12.683a1.715 1.715 0 0 0 1.318-3.165a1.7 1.7 0 0 0-1.263-.02l1.023.424a1.261 1.261 0 1 1-.97 2.33l-.99-.41a1.7 1.7 0 0 0 .882.84Zm3.726-6.687a2.03 2.03 0 0 0 2.027 2.029a2.03 2.03 0 0 0 2.027-2.029a2.03 2.03 0 0 0-2.027-2.027a2.03 2.03 0 0 0-2.027 2.027m2.03-1.527a1.524 1.524 0 1 1-.002 3.048a1.524 1.524 0 0 1 .002-3.048"/></g><g fill="none" fill-rule="evenodd" stroke="#d91111" stroke-linecap="round" stroke-linejoin="round" transform="translate(-0.5 -0.5)" stroke-width="2"><circle cx="8.5" cy="8.5" r="7"/><path d="M12 4L4 12"/></g></svg> No Steam mode

!!! danger "Experimental Feature"

    Using SISR without Steam is an experimental feature and may not work as expected in all scenarios.

SISR can be used without Steam as a general gamepad to gamepad translator.  
Eg. you can map a real Switch/Playstation/Steam Controller to an emulated Xbox360 controller, without the need to have Steam running.  

This feature is primarily intended for [networked usage](./networked.md) scenarios,
for devices that may not be able to run Steam themselves (eg. ARM based machines)

In "no Steam" mode SISR will **not** interact with Steam in any way.  
This means:  

- No Steam Input configuration will be "_forced_"
- Steam Input remappings will **not** be applied _to emulated controllers_
- Steam should™️ not be required to be running at all  
  <sup>(but is fine if it is)</sup>

!!! tip "Launch No-Steam via Steam"
    It sounds counter-intuitive, but you can still launch SISR in "_No Steam_" mode via Steam!  
    In this case only real, physical controllers will be picked up by SISR and any "_Steam Virtual Gamepads_" will be ignored.  

## Enabling DS4 Emulation

To enable No-Steam mode in SISR, pass the `--no-steam=true` launch argument to SISR.  
For permanent configuration see [Configuration](../../../config/config)

!!! warning "Remapping"

    SISR will **not** provide any remapping functionality in No-Steam mode.  
    Nor is this feature planned!  

    You can however use SDL3s built-in remapping functionality via environment variables.  
    See the [SDL3 documentation](https://wiki.libsdl.org/SDL3/SDL_AddGamepadMapping) and corresponding [SDL_Hint](https://wiki.libsdl.org/SDL3/SDL_HINT_GAMECONTROLLERCONFIG) documentation for more information.
