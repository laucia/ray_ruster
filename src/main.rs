extern crate epoxy;
extern crate gio;
extern crate gl;
extern crate gtk;

// Based off of the static-triangle example from gl-rs
mod example {
    extern crate epoxy;
    extern crate gl;
    use gtk::GLArea;
    use ray_ruster::viewer::glscene::GLScene;

    use gtk;
    use gtk::traits::*;
    use gtk::Window;
    use std::sync::Arc;

    use self::gl::types::*;

    pub fn main() {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let window = Window::new(gtk::WindowType::Toplevel);
        let glarea = GLArea::new();
        let vertices: [GLfloat; 15] = [
            0.0, 0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, -0.5, -0.5, 0.0, 0.0, 1.0,
        ];
        let glscene = Arc::new(GLScene::new(&vertices));

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });

        {
            let glarea_realize = glarea.clone();
            let glscene_realize = glscene.clone();
            glarea_realize.connect_realize(move |_widget| {
                _widget.make_current();
                glscene_realize.load_vertices();
                glscene_realize.render();
            });
        }
        {
            let glarea_connect = glarea.clone();
            let glscene_realize = glscene.clone();

            glarea_connect.connect_render(move |_, _| {
                glscene_realize.render();
                gtk::Inhibit(false)
            });
        }

        window.set_title("GLArea Example");
        window.add(&glarea.clone());
        window.set_default_size(400, 400);

        window.show_all();
        gtk::main();
    }
}

fn main() {
    // let application = Application::new(
    //     Some("com.github.gtk-rs.examples.basic"),
    //     Default::default(),
    // ).expect("failed to initialize GTK application");

    // application.connect_activate(|app| {
    //     let window = ApplicationWindow::new(app);
    //     window.set_title("First GTK+ Program");
    //     window.set_default_size(350, 70);

    //     let button = Button::new_with_label("Click me!");
    //     button.connect_clicked(|_| {
    //         println!("Clicked!");
    //     });
    //     window.add(&button);

    //     // create a GtkGLArea instance
    //     let gl_area = GLArea::new() ;

    //     window.add(&gl_area);

    //     window.show_all();
    // });

    // application.run(&[]);

    example::main();
}
