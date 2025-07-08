pub const SAMPLE_TITLES: &str = "Vivid Dreamscape: Dali-esque, Glowing Grove, Lily Luminescence, Geometric Genesis, Diminuendo Dusk";

pub const SAMPLE_PROMPTS: &str = "
 - prompt 1: A painting of a vivid, dreamy, surreal landscape, inspired by the works of Salvador Dali.
 - prompt 2: A digital composition reminiscent of Henri Rousseau's lush jungles, depicting a surreal nocturnal scene where bioluminescent plants and creatures create a symphony of light under the canopy. A tranquil azure pond, reflecting the subtle glow and intertwined vines forming natural arabesques, adds to the dreamlike atmosphere, while a hidden tiger's eyes glimmer with the wisdom of the wild.
 - prompt 3: A photographic interpretation of Monet's 'Water Lilies,' capturing the ephemeral beauty of a pond filled with luminous, blushing pink lily petals lightly dusted with morning dew, the surface broken only by the darting flash of elusive golden-orange koi and mirrored by a dappled canvas of sky, clouds, and overhanging willow branches mirrored on the calm water.
 - prompt 4: A modernist oil painting inspired by Pablo Picasso, showcasing an abstract geometric rendering of a flourishing garden, birthed through the symbiotic dance between botany and geometry. Sharp vertices of emerald-green leaves mingle with rounds of celestial-blue blooms. Crimson rays of a cubist sun uniquely fracture and reshape the landscape whilst sporadic golden-rust pebbles and sapphire rivulets conjugate to bring lucid texture to this spatial dialogue. Sprouting within this fertile collision of shape and form, a single orchid pink geometry expresses itself with defiant beauty elicited from the chaos.
 - prompt 5: A painterly photograph channeling the chiaroscuro intensity of Caravaggio's work to illustrate a modern street violinist lost in the emotion of a nocturne piece, with dramatic highlights cutting through the encroaching shadows of a forgotten alley.";

pub const TITLE_PROMPT: &str = "Create a captivating and imaginative title that goes beyond a literal interpretation.
Avoid using the artist's name, and feel free to incorporate clever wordplay or alliteration inspired by the description.
Here are some titles that you can take some inspiration from (never use these verbatim, they serve as inspiration for you to come up with something original): {{TITLES}}
The title should be no longer than 27 characters (Ideally it should be quite short, between two or three words) and evoke a sense of beauty, emotion, or intrigue.
For the following description (remember to give me *just* the title): {{DESCRIPTION}}.";

pub const IMAGE_PROMPT: &str = "Can you create a prompt for an AI image generator (e.g., DALL-E 2, MidJourney, StableDiffusion) to produce a compelling and artistic image?

The primary goal is an artistic image. Be imaginative and descriptive, but remember simplicity can also be beautiful. Avoid over-saturating with colors or elements if it doesn\'t fit the style.

Specify the type of image: e.g., a photograph, painting, landscape description, unique object, or abstract concept. At least one in ten prompts should be for a photograph. For other types, aim for unique styles, painterly or illustrative qualities rather than strict photorealism.

To inspire style, you can occasionally reference varied painters or photographers, but avoid overusing the same well-known names.

To enhance creativity and originality:
- Steer clear of common clichés (e.g., excessive butterflies, lighthouses, generic ethereal scenes) and aim for fresh concepts.
- Explore a diverse range of subjects, from urban scenes and human emotions to scientific ideas or cultural traditions, rendered artistically.
- Emphasize artistic interpretation: consider different mediums (oil, watercolor, digital, ink), art movements (impressionism, art deco, surrealism - used thoughtfully), dramatic lighting, and expressive color palettes.
- When describing, focus on the scene, style, emotion, and medium directly, rather than generic phrases like \'A painting of X\'. For instance: \'Impressionistic oil sketch of a bustling city square at twilight, capturing the fleeting light on wet cobblestones.\'

Your primary goal is to create a beautiful, compelling, and artistic image prompt. While maintaining high artistic standards, ensure creative diversity by analyzing these previous prompts and deliberately choosing different subjects, styles, mediums, and moods to avoid repetition:

{{PROMPTS}}

Create something artistically excellent that contrasts with the above - if they focus on nature, explore urban beauty; if they use warm colors, try cool palettes; if they're realistic, consider abstract approaches; if they're paintings, try photography or digital art. Always prioritize artistic beauty and emotional impact over mere differentiation.

Please give me *just* the prompt surrounded by single quotes and nothing more before or after it. This is EXTREMELY important. The prompt should be a concise yet descriptive instruction for an image generation AI.";